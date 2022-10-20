use serde_json;
mod types;

wit_bindgen_guest_rust::import!("/Users/shobhitsharma/repos/fifthtry/fpm-utils/wits/host.wit");

#[fpm_utils_macro::wasm_backend]
fn handlerequest(a: guest_backend::Httprequest) -> guest_backend::Httpresponse {
    let base_url = "https://jijweopljiyfrxeolnkt.supabase.co/rest/v1";
    let apikey = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imppandlb3Bsaml5ZnJ4ZW9sbmt0Iiwicm9sZSI6ImFub24iLCJpYXQiOjE2NjMxNTM2NjksImV4cCI6MTk3ODcyOTY2OX0.Urn5gEQyen8Kig-ArlfpP7N4CFktDJCJA1PZDQYYaOg";
    let header_map = [("Content-Type", "application/json"), ("apiKey", apikey)];
    return guest_backend::Httpresponse {
        data: serde_json::to_string(&a).unwrap(),
        success: false,
    };
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
                data: String::from("{\"Hello\": \"World!\"}"),
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
