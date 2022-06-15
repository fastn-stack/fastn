/// FPM Controller Support
/// FPM cli supports communication with fpm controller. This is an optional feature, and is only
/// available when controller feature is enabled, which is not enabled by default.
/// Controller Communication
/// When controller feature is enabled, fpm serve will first communicate with the FPM controller
/// service’s /get-package/ API.

/// FPM Controller Service Endpoint
/// The FPM Controller Service’s endpoint is computed by using environment variable FPM_CONTROLLER,
/// which will look something like this: https://controller.fifthtry.com, with the API path.
/// FPM Controller Service has more than one APIs: /get-package/ and /fpm-ready/.

/// get-package:
/// Through an environment variable FPM_INSTANCE_ID, the fpm serve will learn it’s instance id, and
/// it will pass the instance id to the get-package API.
/// The API returns the URL of the package to be downloaded, git repository URL and the package name.
/// FPM will clone the git repository in the current directory. The current directory will contain
/// FPM.ftd and other files of the package.
/// FPM will then calls fpm install on it.

/// fpm-ready:
/// Once dependencies are ready fpm calls /fpm-ready/ API on the controller. We will pass the
/// FPM_INSTANCE_ID and the git commit hash as input to the API
/// The API will return with success, and once it is done fpm will start receiving HTTP traffic
/// from the controller service.

#[derive(serde::Deserialize, Debug)]
struct ApiResponse<T> {
    success: bool,
    result: Option<T>,
    message: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct PackageResult {
    package: String,
    git: String,
}

pub async fn resolve_dependencies(fpm_instance: String, fpm_controller: String) -> fpm::Result<()> {
    // First call get_package API to get package details and resolve dependencies

    // response from get-package API
    let package_response = get_package(fpm_instance.as_str(), fpm_controller.as_str()).await?;

    // Clone the git package into the current directory
    // Need to execute shell commands from rust
    // git_url https format: https://github.com/<user>/<repo>.git

    let package =
        fpm::Package::new(package_response.package.as_str()).with_zip(package_response.git);

    package.unzip_package().await?;
    fpm::Config::read(None).await?;

    /*let out = std::process::Command::new("git")
           .arg("clone")
           .arg(git_url)
           .output()
           .expect("unable to execute git clone command");

    if out.status.success() {
    // By this time the cloned repo should be available in the current directory
    println!("Git cloning successful for the package {}", package_name);
    // Resolve dependencies by reading the FPM.ftd using config.read()
    // Assuming package_name and repo name are identical
    let _config = fpm::Config::read(Some(package_name.to_string())).await?;
    }*/

    // Once the dependencies are resolved for the package
    // then call fpm_ready API to ensure that the controller service is now ready

    // response from fpm_ready API

    fpm_ready(fpm_instance.as_str(), fpm_controller.as_str()).await?;

    Ok(())
}

/// get-package API
/// input: fpm_instance
/// output: package_name and git repo URL
/// format: {
///     "success": true,
///     "result": {
///         "package": "<package name>"
///         "git": "<git url>"
///     }
/// }
async fn get_package(fpm_instance: &str, fpm_controller: &str) -> fpm::Result<PackageResult> {
    let controller_api = format!(
        "{}/v1/fpm/get-package?ec2_reservation={}",
        fpm_controller, fpm_instance
    );

    let url = url::Url::parse(controller_api.as_str())?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fpm"),
    );

    let resp: ApiResponse<PackageResult> = fpm::library::http::get_with_type(url, headers).await?;

    if !resp.success {
        return Err(fpm::Error::APIResponseError(format!(
            "get_package api error: {:?}",
            resp.message
        )));
    }

    resp.result.ok_or({
        fpm::Error::APIResponseError(format!("get_package api error: {:?}", &resp.message))
    })
}

/// fpm-ready API
/// input: fpm_instance, *(git commit hash)
/// output: success: true/false
/// format: lang: json
/// {
///     "success": true
/// }

/// Git commit hash needs to be computed before making a call to the fpm_ready API
async fn fpm_ready(fpm_instance: &str, fpm_controller: &str) -> fpm::Result<()> {
    let git_commit = "<dummy-git-commit-hash-xxx123>";

    let controller_api = format!(
        "{}/v1/fpm/fpm-ready?ec2_reservation={}&hash={}",
        fpm_controller, fpm_instance, git_commit
    );

    let url = url::Url::parse(controller_api.as_str())?;

    // This request should be put request for fpm_ready API to update the instance status to ready
    // Using http::_get() function to make request to this API for now
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fpm"),
    );

    // TODO: here Map is wrong,
    let resp: ApiResponse<std::collections::HashMap<String, String>> =
        fpm::library::http::get_with_type(url, headers).await?;
    if !resp.success {
        return Err(fpm::Error::APIResponseError(format!(
            "fpm_ready api error: {:?}",
            resp.message
        )));
    }
    Ok(())
}
