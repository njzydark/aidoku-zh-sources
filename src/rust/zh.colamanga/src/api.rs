use aidoku::{
	error::{AidokuError, Result},
	helpers::uri::encode_uri_component,
	prelude::{format, println},
	std::{
		net::{HttpMethod, Request},
		ObjectRef, String, StringRef,
	},
};
use alloc::{borrow::ToOwned, vec::Vec};

const BASE_URL: &str = "http://localhost:8346";

#[derive(Debug)]
pub struct PageListRes {
	page_total_count: i64,
	page_img_list: Vec<Result<StringRef>>,
	key_type: String,
	img_key: String,
}

pub fn get_page_list(base_data: &str) -> Result<Vec<String>> {
	let url = format!("{}/pages/{}", BASE_URL, base_data);
	let raw_res = Request::get(url.as_str()).json()?.as_object()?;

	let page_list_res = PageListRes {
		page_total_count: raw_res.get("pageTotalCount").as_int()?,
		key_type: raw_res.get("keyType").as_string()?.read(),
		img_key: raw_res.get("imgKey").as_string()?.read(),
		page_img_list: raw_res
			.get("pageImgList")
			.as_array()?
			.map(|src| src.as_string())
			.collect(),
	};

	let pages: Vec<String> = page_list_res
		.page_img_list
		.into_iter()
		.map(|src| {
			format!(
				"{}/img?src={}&imgKey={}&keyType={}",
				BASE_URL,
				encode_uri_component(&src.unwrap_or_default().read()),
				page_list_res.img_key,
				page_list_res.key_type
			)
		})
		.collect();

	println!("{:?}", pages);

	Ok(pages)
}
