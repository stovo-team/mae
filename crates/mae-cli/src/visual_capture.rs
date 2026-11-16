use std::fs;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use mae_domain::DomainError;

const DEFAULT_SOURCE_URL: &str = "http://127.0.0.1:5173";
const MAX_PIXEL_DELTA_RATIO: f64 = 0.02;

pub fn verify_live_source(source_dir: &Path, target_dir: &Path) -> Result<String, DomainError> {
    let baseline_path = source_dir.join("runtime-baseline.json");
    let baseline = fs::read_to_string(&baseline_path).map_err(|error| {
        DomainError::PortError(format!("Cannot read {}: {error}", baseline_path.display()))
    })?;
    let routes = extract_observed_routes(&baseline)?;
    let package_path = source_dir.join("package.json");
    let package = fs::read_to_string(&package_path).map_err(|error| {
        DomainError::PortError(format!("Cannot read {}: {error}", package_path.display()))
    })?;
    let script = discover_source_script(&package).ok_or_else(|| {
        DomainError::PortError(format!(
            "Visual verification requires a `mock` or `dev` script in {}",
            package_path.display()
        ))
    })?;

    let source_url = std::env::var("MAE_SOURCE_URL").unwrap_or_else(|_| DEFAULT_SOURCE_URL.into());
    let endpoint = endpoint_from_url(&source_url)?;
    let port = endpoint
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok())
        .ok_or_else(|| {
            DomainError::PortError(format!("Cannot find port in source URL {source_url}"))
        })?;
    let mut server = Server(
        Command::new("npm")
            .args([
                "run",
                &script,
                "--",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
            ])
            .current_dir(source_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                DomainError::PortError(format!("Cannot start npm run {script}: {error}"))
            })?,
    );
    wait_for_server(&mut server.0, &endpoint)?;

    let output_root = target_dir.join(".mae-visual");
    let source_shots = output_root.join("source");
    let native_shots = output_root.join("native");
    fs::create_dir_all(&source_shots).map_err(io_error)?;
    let chromium = std::env::var("MAE_CHROMIUM").unwrap_or_else(|_| "chromium".into());
    let mut failures = Vec::new();
    for route in &routes {
        let name = route_file_name(route);
        let source_image = source_shots.join(format!("{name}.png"));
        let native_image = native_shots.join(format!("{name}.png"));
        let url = format!("{}{}", source_url.trim_end_matches('/'), route);
        let capture = Command::new(&chromium)
            .args([
                "--headless=new",
                "--no-sandbox",
                "--disable-gpu",
                "--hide-scrollbars",
                "--force-device-scale-factor=1",
                "--window-size=1280,800",
            ])
            .arg(format!("--screenshot={}", source_image.display()))
            .arg(url)
            .output()
            .map_err(|error| DomainError::PortError(format!("Cannot run {chromium}: {error}")))?;
        if !capture.status.success() || !source_image.exists() {
            failures.push(format!("{route}: Chromium reference capture failed"));
            continue;
        }
        if !native_image.exists() {
            failures.push(format!(
                "{route}: missing native screenshot {}; launch/capture the target for this route first",
                native_image.display()
            ));
            continue;
        }
        match compare_images(&source_image, &native_image) {
            Ok(ratio) if ratio <= MAX_PIXEL_DELTA_RATIO => {}
            Ok(ratio) => failures.push(format!(
                "{route}: pixel delta {ratio:.4} exceeds {MAX_PIXEL_DELTA_RATIO:.4}"
            )),
            Err(error) => failures.push(format!("{route}: {error}")),
        }
    }
    drop(server);
    if failures.is_empty() {
        Ok(format!(
            "visual_status=PASS, routes={}, max_pixel_delta={MAX_PIXEL_DELTA_RATIO:.4}",
            routes.len()
        ))
    } else {
        Err(DomainError::PortError(format!(
            "Visual parity failed for {} of {} routes: {}. Reference screenshots: {}",
            failures.len(),
            routes.len(),
            failures.join("; "),
            source_shots.display()
        )))
    }
}

fn extract_observed_routes(source: &str) -> Result<Vec<String>, DomainError> {
    let key = source.find("\"observed_routes\"").ok_or_else(|| {
        DomainError::PortError("runtime-baseline.json is missing observed_routes".into())
    })?;
    let list = &source[key..];
    let start = list
        .find('[')
        .ok_or_else(|| DomainError::PortError("observed_routes must be an array".into()))?;
    let end = list[start + 1..]
        .find(']')
        .map(|i| start + 1 + i)
        .ok_or_else(|| DomainError::PortError("observed_routes array is not closed".into()))?;
    let body = &list[start + 1..end];
    let routes = body
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    if routes.is_empty() || routes.iter().any(|route| !route.starts_with('/')) {
        return Err(DomainError::PortError(
            "observed_routes must contain absolute paths".into(),
        ));
    }
    Ok(routes)
}

fn discover_source_script(package: &str) -> Option<String> {
    let scripts_at = package.find("\"scripts\"")?;
    let scripts = &package[scripts_at..];
    let object_end = scripts.find('}')?;
    let scripts = &scripts[..=object_end];
    ["mock", "dev"]
        .into_iter()
        .find(|name| scripts.contains(&format!("\"{name}\"")))
        .map(str::to_owned)
}

fn endpoint_from_url(url: &str) -> Result<String, DomainError> {
    let authority = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("");
    if authority.is_empty() {
        return Err(DomainError::PortError(format!("Invalid source URL {url}")));
    }
    let host_port = if authority.contains(':') {
        authority.to_string()
    } else {
        format!("{authority}:80")
    };
    Ok(host_port)
}

fn wait_for_server(child: &mut Child, endpoint: &str) -> Result<(), DomainError> {
    let address = endpoint
        .to_socket_addrs()
        .map_err(io_error)?
        .next()
        .ok_or_else(|| {
            DomainError::PortError(format!("Cannot resolve source server {endpoint}"))
        })?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().map_err(io_error)? {
            return Err(DomainError::PortError(format!(
                "Source dev server exited early: {status}"
            )));
        }
        if let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(250)) {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
            let request =
                format!("GET / HTTP/1.1\r\nHost: {endpoint}\r\nConnection: close\r\n\r\n");
            if stream.write_all(request.as_bytes()).is_ok() {
                let mut response = String::new();
                if stream.read_to_string(&mut response).is_ok()
                    && response
                        .lines()
                        .next()
                        .is_some_and(|line| line.contains(" 200 "))
                {
                    return Ok(());
                }
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
    Err(DomainError::PortError(format!(
        "Source server healthcheck timed out at {endpoint}"
    )))
}

fn compare_images(source: &Path, target: &Path) -> Result<f64, DomainError> {
    let identify = Command::new("identify")
        .args([
            "-format",
            "%w %h\n",
            source.to_str().unwrap_or_default(),
            target.to_str().unwrap_or_default(),
        ])
        .output()
        .map_err(|error| {
            DomainError::PortError(format!("Cannot run ImageMagick identify: {error}"))
        })?;
    if !identify.status.success() {
        return Err(DomainError::PortError(
            "ImageMagick could not read a screenshot".into(),
        ));
    }
    let dimensions = String::from_utf8_lossy(&identify.stdout)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if dimensions.len() != 4 || dimensions[0] != dimensions[2] || dimensions[1] != dimensions[3] {
        return Err(DomainError::PortError(
            "Source and native screenshots have different dimensions".into(),
        ));
    }
    let width = dimensions[0]
        .parse::<u64>()
        .map_err(|_| DomainError::PortError("Invalid screenshot width".into()))?;
    let height = dimensions[1]
        .parse::<u64>()
        .map_err(|_| DomainError::PortError("Invalid screenshot height".into()))?;
    let comparison = Command::new("compare")
        .args([
            "-metric",
            "AE",
            source.to_str().unwrap_or_default(),
            target.to_str().unwrap_or_default(),
            "null:",
        ])
        .output()
        .map_err(|error| {
            DomainError::PortError(format!("Cannot run ImageMagick compare: {error}"))
        })?;
    let metric = String::from_utf8_lossy(&comparison.stderr);
    let differing_pixels = metric
        .split_whitespace()
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| {
            DomainError::PortError(format!("Cannot parse ImageMagick pixel delta: {metric}"))
        })?;
    image_delta_ratio(differing_pixels, width, height)
}

fn image_delta_ratio(differing_pixels: u64, width: u64, height: u64) -> Result<f64, DomainError> {
    let total = width
        .checked_mul(height)
        .filter(|count| *count > 0)
        .ok_or_else(|| DomainError::PortError("Invalid screenshot size".into()))?;
    if differing_pixels > total {
        return Err(DomainError::PortError(
            "Pixel delta exceeds image dimensions".into(),
        ));
    }
    Ok(differing_pixels as f64 / total as f64)
}

fn route_file_name(route: &str) -> String {
    if route == "/" {
        return "index".into();
    }
    route.trim_matches('/').replace(['/', '-'], "_")
}

fn io_error(error: std::io::Error) -> DomainError {
    DomainError::PortError(error.to_string())
}

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::{discover_source_script, extract_observed_routes, image_delta_ratio};
    use std::path::Path;

    #[test]
    fn reads_observed_routes_from_runtime_baseline() {
        let routes =
            extract_observed_routes(r#"{"observed_routes":["/","/tasks","/settings"]}"#).unwrap();
        assert_eq!(routes, ["/", "/tasks", "/settings"]);
    }

    #[test]
    fn prefers_mock_script_and_falls_back_to_dev() {
        let package = r#"{"scripts":{"dev":"vite","mock":"vite --mode mock"}}"#;
        assert_eq!(discover_source_script(package).as_deref(), Some("mock"));
        assert_eq!(
            discover_source_script(r#"{"scripts":{"dev":"vite"}}"#).as_deref(),
            Some("dev")
        );
        assert_eq!(
            discover_source_script(r#"{"scripts":{"build":"tsc"}}"#),
            None
        );
    }

    #[test]
    fn computes_normalized_pixel_difference_ratio() {
        assert_eq!(image_delta_ratio(80, 100, 100).unwrap(), 0.008);
        assert!(image_delta_ratio(10_001, 100, 100).is_err());
    }

    #[test]
    fn reports_missing_source_server_script_instead_of_skipping_visual_check() {
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/react-rust-slint/source");
        let target = std::env::temp_dir().join("mae-visual-no-server-target");
        let error = super::verify_live_source(&source, &target).unwrap_err();
        assert!(error.to_string().contains("mock` or `dev` script"));
    }

    #[test]
    fn verify_visual_cli_option_runs_the_live_capture_gate() {
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/react-rust-slint/source");
        let target = std::env::temp_dir().join("mae-visual-cli-target");
        let error = crate::execute_cli_args(&[
            "mae".into(),
            "verify".into(),
            "--visual".into(),
            target.display().to_string(),
            source.display().to_string(),
        ])
        .unwrap_err();
        assert!(error.to_string().contains("mock` or `dev` script"));
    }
}
