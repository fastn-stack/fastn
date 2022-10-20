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
/// The API will return with api_ok, and once it is done fpm will start receiving HTTP traffic
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

pub async fn resolve_dependencies(fpm_instance: String, fpm_controller: String) -> fpm::Result<()> {
    // First call get_package API to get package details and resolve dependencies

    // response from get-package API
    println!("getting package from fpm controller");
    let package_response = get_package(fpm_instance.as_str(), fpm_controller.as_str()).await?;

    // Clone the git package into the current directory
    // Need to execute shell commands from rust
    // git_url https format: https://github.com/<user>/<repo>.git

    // let package =
    //     fpm::Package::new(package_response.package.as_str()).with_base(package_response.base);
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
        Err(e) => return Err(fpm::Error::APIResponseError(e.to_string())),
    };

    if out.status.success() {
        // By this time the cloned repo should be available in the current directory
        println!(
            "Git cloning successful for the package {}",
            package_response.package
        );
        // Resolve dependencies by reading the FPM.ftd using config.read()
        // Assuming package_name and repo name are identical
        fpm::Config::read(None, false, None).await?;
    } else {
        return Err(fpm::Error::APIResponseError(format!(
            "Package {} Cloning failed: {}",
            package_response.package,
            String::from_utf8(out.stderr)?
        )));
    }

    // Once the dependencies are resolved for the package
    // then call fpm_ready API to ensure that the controller service is now ready

    // response from fpm_ready API

    println!("calling fpm ready");
    fpm_ready(fpm_instance.as_str(), fpm_controller.as_str()).await?;

    Ok(())
}

/// get-package API
/// input: fpm_instance
/// output: package_name and git repo URL
/// format: {
///     "api_ok": true,
///     "result": {
///         "package": "<package name>"
///         "git": "<git url>"
///     }
/// }
async fn get_package(fpm_instance: &str, fpm_controller: &str) -> fpm::Result<PackageResult> {
    let controller_api = format!(
        "{}/v1/fpm/get-package?ec2_instance_id={}",
        fpm_controller, fpm_instance
    );

    let resp: ApiResponse<PackageResult> = crate::http::get_json(controller_api.as_str()).await?;
    if !resp.success {
        return Err(fpm::Error::APIResponseError(format!(
            "get_package api error: {:?}",
            resp.message
        )));
    }

    #[allow(clippy::or_fun_call)]
    resp.result.ok_or({
        fpm::Error::APIResponseError(format!("get_package api error: {:?}", &resp.message))
    })
}

/// fpm-ready API
/// input: fpm_instance, *(git commit hash)
/// output: api_ok: true/false
/// format: lang: json
/// {
///     "api_ok": true
/// }

/// Git commit hash needs to be computed before making a call to the fpm_ready API
async fn fpm_ready(fpm_instance: &str, fpm_controller: &str) -> fpm::Result<()> {
    let latest_commit = || -> fpm::Result<String> {
        let out = match std::process::Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .output()
        {
            Ok(output) => output,
            Err(e) => return Err(fpm::Error::APIResponseError(e.to_string())),
        };

        if !out.status.success() {
            // By this time the cloned repo should be available in the current directory
            println!("Git cloning successful for the package",);
            return Err(fpm::Error::APIResponseError(String::from_utf8(out.stderr)?));
        }

        Ok(String::from_utf8(out.stdout)?.trim().to_string())
    };

    let controller_api = format!(
        "{}/v1/fpm/fpm-ready?ec2_instance_id={}&hash={}",
        fpm_controller,
        fpm_instance,
        latest_commit()?
    );

    let url = url::Url::parse(controller_api.as_str())?;

    // This request should be put request for fpm_ready API to update the instance status to ready
    // Using http::_get() function to make request to this API for now
    // TODO: here Map is wrong,
    let resp: ApiResponse<std::collections::HashMap<String, String>> =
        crate::http::get_json(url.as_str()).await?;

    if !resp.success {
        return Err(fpm::Error::APIResponseError(format!(
            "fpm_ready api error: {:?}",
            resp.message
        )));
    }
    Ok(())
}

// This API will be called from can_read and can_write functions

pub async fn get_remote_identities(
    remote_host: &str,
    cookies: &std::collections::HashMap<String, String>,
    identities: &[(String, String)],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;

    #[derive(serde::Deserialize)]
    struct UserIdentities {
        success: bool,
        reason: Option<String>,
        #[serde(rename = "user-identities")]
        user_identities: Option<Vec<std::collections::HashMap<String, String>>>,
    }
    let url = format!("https://{}/-/dj/get-identities/", remote_host);
    println!("remote url: {}", url);

    let cookie = cookies
        .iter()
        .map(|c| format!("{}={}", c.0, c.1))
        .collect::<Vec<_>>()
        .join(";");

    println!("cookies: {}", cookie);

    let headers = reqwest::header::HeaderMap::from_iter([(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_bytes(cookie.as_bytes()).unwrap(),
    )]);

    let resp: UserIdentities =
        crate::http::http_get_with_type(url::Url::parse(url.as_str())?, headers, identities)
            .await?;

    if !resp.success {
        return Err(fpm::Error::APIResponseError(format!(
            "url: {}, error while getting controller: get-identities",
            url,
        )));
    }

    #[allow(clippy::or_fun_call)]
    let remote_identities = resp.user_identities.ok_or({
        fpm::Error::APIResponseError(format!(
            "controller:get-identities api error: {:?}",
            resp.reason
        ))
    })?;

    Ok(remote_identities
        .into_iter()
        .flat_map(|identities| {
            identities
                .into_iter()
                .map(|(key, value)| fpm::user_group::UserIdentity { key, value })
        })
        .collect_vec())
}
