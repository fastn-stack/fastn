/// fastn Controller Support
/// fastn cli supports communication with fastn controller. This is an optional feature, and is only
/// available when controller feature is enabled, which is not enabled by default.
/// Controller Communication
/// When controller feature is enabled, fastn serve will first communicate with the fastn controller
/// service’s /get-package/ API.

/// fastn Controller Service Endpoint
/// The fastn Controller Service’s endpoint is computed by using environment variable fastn_CONTROLLER,
/// which will look something like this: https://controller.fifthtry.com, with the API path.
/// fastn Controller Service has more than one APIs: /get-package/ and /fastn-ready/.

/// get-package:
/// Through an environment variable fastn_INSTANCE_ID, the fastn serve will learn it’s instance id, and
/// it will pass the instance id to the get-package API.
/// The API returns the URL of the package to be downloaded, git repository URL and the package name.
/// fastn will clone the git repository in the current directory. The current directory will contain
/// FASTN.ftd and other files of the package.
/// fastn will then calls fastn install on it.

/// fastn-ready:
/// Once dependencies are ready fastn calls /fastn-ready/ API on the controller. We will pass the
/// fastn_INSTANCE_ID and the git commit hash as input to the API
/// The API will return with api_ok, and once it is done fastn will start receiving HTTP traffic
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
    #[allow(dead_code)]
    base: String,
    git: String,
}

pub async fn resolve_dependencies(
    fastn_instance: String,
    fastn_controller: String,
) -> fastn_core::Result<()> {
    // First call get_package API to get package details and resolve dependencies

    // response from get-package API
    println!("getting package from fastn controller");
    let package_response = get_package(fastn_instance.as_str(), fastn_controller.as_str()).await?;

    // Clone the git package into the current directory
    // Need to execute shell commands from rust
    // git_url https format: https://github.com/<user>/<repo>.git

    // let package =
    //     fastn_core::Package::new(package_response.package.as_str()).with_base(package_response.base);
    //
    // package.unzip_package().await?;

    println!("Cloning git repository...");
    let out = match std::process::Command::new("git")
        .arg("clone")
        .args(["--depth", "1"])
        .arg(package_response.git)
        .arg(".")
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(fastn_core::Error::APIResponseError(e.to_string())),
    };

    if out.status.success() {
        // By this time the cloned repo should be available in the current directory
        println!(
            "Git cloning successful for the package {}",
            package_response.package
        );
        // Resolve dependencies by reading the FASTN.ftd using config.read()
        // Assuming package_name and repo name are identical
        fastn_core::Config::read(None, false, None).await?;
    } else {
        return Err(fastn_core::Error::APIResponseError(format!(
            "Package {} Cloning failed: {}",
            package_response.package,
            String::from_utf8(out.stderr)?
        )));
    }

    // Once the dependencies are resolved for the package
    // then call fastn_ready API to ensure that the controller service is now ready

    // response from fastn_ready API

    println!("calling fastn ready");
    fastn_ready(fastn_instance.as_str(), fastn_controller.as_str()).await?;

    Ok(())
}

/// get-package API
/// input: fastn_instance
/// output: package_name and git repo URL
/// format: {
///     "api_ok": true,
///     "result": {
///         "package": "<package name>"
///         "git": "<git url>"
///     }
/// }
async fn get_package(fastn_instance: &str, fastn_controller: &str) -> fastn_core::Result<PackageResult> {
    let controller_api = format!(
        "{}/v1/fastn/get-package?ec2_instance_id={}",
        fastn_controller, fastn_instance
    );

    let resp: ApiResponse<PackageResult> = crate::http::get_json(controller_api.as_str()).await?;
    if !resp.success {
        return Err(fastn_core::Error::APIResponseError(format!(
            "get_package api error: {:?}",
            resp.message
        )));
    }

    #[allow(clippy::or_fun_call)]
    resp.result.ok_or({
        fastn_core::Error::APIResponseError(format!("get_package api error: {:?}", &resp.message))
    })
}

/// fastn-ready API
/// input: fastn_instance, *(git commit hash)
/// output: api_ok: true/false
/// format: lang: json
/// {
///     "api_ok": true
/// }

/// Git commit hash needs to be computed before making a call to the fastn_ready API
async fn fastn_ready(fastn_instance: &str, fastn_controller: &str) -> fastn_core::Result<()> {
    let latest_commit = || -> fastn_core::Result<String> {
        let out = match std::process::Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .output()
        {
            Ok(output) => output,
            Err(e) => return Err(fastn_core::Error::APIResponseError(e.to_string())),
        };

        if !out.status.success() {
            // By this time the cloned repo should be available in the current directory
            println!("Git cloning successful for the package",);
            return Err(fastn_core::Error::APIResponseError(String::from_utf8(
                out.stderr,
            )?));
        }

        Ok(String::from_utf8(out.stdout)?.trim().to_string())
    };

    let controller_api = format!(
        "{}/v1/fastn/fastn-ready?ec2_instance_id={}&hash={}",
        fastn_controller,
        fastn_instance,
        latest_commit()?
    );

    let url = url::Url::parse(controller_api.as_str())?;

    // This request should be put request for fastn_ready API to update the instance status to ready
    // Using http::_get() function to make request to this API for now
    // TODO: here Map is wrong,
    let resp: ApiResponse<std::collections::HashMap<String, String>> =
        crate::http::get_json(url.as_str()).await?;

    if !resp.success {
        return Err(fastn_core::Error::APIResponseError(format!(
            "fastn_ready api error: {:?}",
            resp.message
        )));
    }
    Ok(())
}
