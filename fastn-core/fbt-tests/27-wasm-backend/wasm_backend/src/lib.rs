use serde_json;
mod types;

wit_bindgen_guest_rust::import!("/Users/shobhitsharma/repos/fifthtry/fpm-utils/wits/host.wit");

#[fpm_utils_macro::wasm_backend]
fn handlerequest(a: guest_backend::Httprequest) -> guest_backend::Httpresponse {
    let base_url_header_key = String::from("X-FPM-BLOG-APP-SUPABASE-BASE-URL");
    let apikey_header_key = String::from("X-FPM-BLOG-APP-SUPABASE-API-KEY");
    let (_, base_url) = a
        .headers
        .iter()
        .find(|(key, _)| key == &base_url_header_key)
        .expect(
            format!(
                "{base_url_header_key} not found in the request. Please configure app properly"
            )
            .as_str(),
        );
    let (_, apikey) = a
        .headers
        .iter()
        .find(|(key, _)| key == &apikey_header_key)
        .expect(
            format!("{apikey_header_key} not found in the request. Please configure app properly")
                .as_str(),
        );
    let header_map = [("Content-Type", "application/json"), ("apiKey", apikey)];
    let resp = match a.path.as_str() {
        "/-/blog-backend.fpm.local/subscribe/" => {
            let data: types::SubscribeData = serde_json::from_str(a.payload.as_str()).unwrap();
            host::http(host::Httprequest {
                path: format!("{base_url}/blog-subscription").as_str(),
                method: "POST",
                payload: serde_json::to_string(&data).unwrap().as_str(),
                headers: &header_map,
            })
        }
        "/-/blog-backend.fpm.local/like/" => {
            let data: types::LikeData = serde_json::from_str(a.payload.as_str()).unwrap();
            host::http(host::Httprequest {
                path: format!("{base_url}/blog-like").as_str(),
                method: "POST",
                payload: serde_json::to_string(&data).unwrap().as_str(),
                headers: &header_map,
            })
        }
        "/-/blog-backend.fpm.local/echo/" => {
            return guest_backend::Httpresponse {
                data: serde_json::to_string(&a).unwrap(),
                success: true,
            };
        }
        x => {
            return guest_backend::Httpresponse {
                data: format!("Route not implemented {x}"),
                success: false,
            }
        }
    };
    guest_backend::Httpresponse {
        data: resp.data,
        success: true,
    }
}
