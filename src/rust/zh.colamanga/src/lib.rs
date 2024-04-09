#![no_std]
extern crate alloc;

mod api;
mod decoder;
mod parser;

use aidoku::{
	error::Result, prelude::*, std::net::HttpMethod, std::net::Request, std::String, std::Vec,
	Chapter, Filter, Manga, MangaPageResult, Page,
};

const BASE_URL: &str = "https://www.colamanga.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut url = String::new();

	parser::get_filtered_url(filters, page, &mut url);

	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	if url.contains("https://www.colamanga.com/show") {
		return parser::parse_home_page(html, &page);
	}
	parser::parse_search_page(html, &page)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = format!("{}/manga-{}/", BASE_URL, id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::parse_manga_details(html, id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/manga-{}/", BASE_URL, id);
	let html = Request::new(url.as_str(), HttpMethod::Get).html()?;
	parser::get_chapter_list(html)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	println!("{} {}", manga_id, chapter_id);
	let base_url = format!("{}/manga-{}/{}.html", BASE_URL, manga_id, chapter_id);
	parser::get_page_list(base_url)
}

#[modify_image_request]
pub fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}
