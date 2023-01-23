pub trait CheckMap {
    fn check_and_insert(&mut self, key: &str, value: ftd::node::Value);
    fn upsert(&mut self, key: &str, value: ftd::node::Value);
}

impl CheckMap for ftd::Map<ftd::node::Value> {
    fn check_and_insert(&mut self, key: &str, value: ftd::node::Value) {
        let value = if let Some(old_value) = self.get(key) {
            let mut new_value = old_value.to_owned();
            if let Some(default) = value.value {
                new_value.value = Some(default);
                new_value.line_number = value.line_number.or(old_value.line_number);
            }
            new_value.properties.extend(value.properties);
            new_value
        } else {
            value
        };

        if value.value.is_some() || !value.properties.is_empty() {
            self.insert(key.to_string(), value);
        }
    }

    fn upsert(&mut self, key: &str, value: ftd::node::Value) {
        if value.value.is_some() || !value.properties.is_empty() {
            self.insert(key.to_string(), value);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn get_translate(
    left: &Option<ftd::executor::TranslateLength>,
    right: &Option<ftd::executor::TranslateLength>,
    up: &Option<ftd::executor::TranslateLength>,
    down: &Option<ftd::executor::TranslateLength>,
    scale: &Option<f64>,
    scale_x: &Option<f64>,
    scale_y: &Option<f64>,
    rotate: &Option<i64>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<Option<String>> {
    let mut translate = match (left, right, up, down) {
        (Some(_), Some(_), Some(_), Some(_)) => {
            return ftd::p2::utils::e2(
                "move-up, move-down, move-left and move-right all 4 can't be used at once!",
                doc_id,
                line_number,
            );
        }
        (Some(_), Some(_), _, _) => {
            return ftd::p2::utils::e2(
                "move-left, move-right both can't be used at once!",
                doc_id,
                line_number,
            );
        }
        (_, _, Some(_), Some(_)) => {
            return ftd::p2::utils::e2(
                "move-up, move-down both can't be used at once!",
                doc_id,
                line_number,
            );
        }
        (Some(l), None, None, None) => Some(format!("translateX(-{}) ", l.to_css_string())),
        (Some(l), None, Some(u), None) => Some(format!("translate(-{}, -{}) ", l.to_css_string(), u.to_css_string())),
        (Some(l), None, None, Some(d)) => Some(format!("translate(-{}, {}) ", l.to_css_string(), d.to_css_string())),
        (None, Some(r), None, None) => Some(format!("translateX({}) ", r.to_css_string())),
        (None, Some(r), Some(u), None) => Some(format!("translate({}, -{}) ", r.to_css_string(), u.to_css_string())),
        (None, Some(r), None, Some(d)) => Some(format!("translate({}, {}) ", r.to_css_string(), d.to_css_string())),
        (None, None, Some(u), None) => Some(format!("translateY(-{}) ", u.to_css_string())),
        (None, None, None, Some(d)) => Some(format!("translateY({}) ", d.to_css_string())),
        _ => None,
    };

    if let Some(ref scale) = scale {
        if let Some(d) = translate {
            translate = Some(format!("{} scale({})", d, scale));
        } else {
            translate = Some(format!("scale({})", scale));
        };
    }
    if let Some(ref scale) = scale_x {
        if let Some(d) = translate {
            translate = Some(format!("{} scaleX({})", d, scale));
        } else {
            translate = Some(format!("scaleX({})", scale));
        };
    }
    if let Some(ref scale) = scale_y {
        if let Some(d) = translate {
            translate = Some(format!("{} scaleY({})", d, scale));
        } else {
            translate = Some(format!("scaleY({})", scale));
        };
    }
    if let Some(ref rotate) = rotate {
        if let Some(d) = translate {
            translate = Some(format!("{} rotate({}deg)", d, rotate));
        } else {
            translate = Some(format!("rotate({}deg)", rotate));
        };
    }
    Ok(translate)
}

pub(crate) fn wrap_to_css(wrap: bool) -> String {
    if wrap {
        "wrap".to_string()
    } else {
        "nowrap".to_string()
    }
}

pub(crate) fn escape(s: &str) -> String {
    let s = s.replace('>', "\\u003E");
    let s = s.replace('<', "\\u003C");
    s.replace('&', "\\u0026")
}

pub(crate) fn count_children_with_absolute_parent(children: &[ftd::executor::Element]) -> usize {
    children
        .iter()
        .filter(|v| {
            let mut bool = false;
            if let Some(common) = v.get_common() {
                if Some(ftd::executor::Anchor::Parent) == common.anchor.value {
                    bool = true;
                }
            }
            bool
        })
        .count()
}
