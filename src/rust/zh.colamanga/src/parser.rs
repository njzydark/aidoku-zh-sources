use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	prelude::*,
	std::html::Node,
	std::Vec,
	std::{net::HttpMethod, net::Request, String},
	Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use alloc::{string::ToString, vec};
use regex::Regex;

use crate::api;

const BASE_URL: &str = "https://www.colamanga.com";

const FILTER_GENRE: [&str; 20] = [
	"all", "10023", "10024", "10126", "10210", "10143", "10124", "10129", "10242", "10560",
	"10122", "10641", "10309", "10461", "11224", "10201", "10943", "10138", "10321", "10301",
];
const FILTER_PROGRESS: [&str; 3] = ["all", "1", "2"];
const SORT: [&str; 4] = ["weeklyCount", "dailyCount", "monthlyCount", "update"];

pub fn parse_home_page(html: Node, page: &i32) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	let ul = "ul.fed-list-info > li";

	for element in html.select(ul).array() {
		let elem = match element.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let manga_id = elem
			.select("a.fed-list-title")
			.attr("href")
			.read()
			.replace("/manga-", "")
			.replace('/', "");
		let title = elem.select("a.fed-list-title").text().read();
		let cover = elem.select("a.fed-list-pics").attr("data-original").read();
		let manga = Manga {
			id: manga_id.clone(),
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: format!("{}/manga-{}/", BASE_URL, manga_id),
			categories: vec![],
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
		};
		mangas.push(manga);
	}

	let mut has_next: bool = false;
	for page_el in html.select(".fed-page-info > a").array() {
		let page_node = match page_el.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		if page_node.text().read() == "尾页"
			&& !page_node
				.attr("onclick")
				.read()
				.contains(&format!("show('{}')", page.to_string()))
		{
			has_next = true;
			break;
		}
	}

	html.close();

	Ok(MangaPageResult {
		manga: mangas,
		has_more: has_next,
	})
}

pub fn parse_search_page(html: Node, page: &i32) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	let ul = "dl.fed-deta-info";

	for element in html.select(ul).array() {
		let elem = match element.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		let manga_id = elem
			.select("a.fed-list-pics")
			.attr("href")
			.read()
			.replace("/manga-", "")
			.replace('/', "");
		let title = elem.select("h1.fed-part-eone a").text().read();
		let cover = elem.select("a.fed-list-pics").attr("data-original").read();
		let description = elem
			.select("li > .fed-part-esan > span")
			.last()
			.text()
			.read();
		let manga = Manga {
			id: manga_id.clone(),
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description,
			url: format!("{}/manga-{}/", BASE_URL, manga_id),
			categories: vec![],
			status: MangaStatus::Completed,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
		};
		mangas.push(manga);
	}

	let mut has_next: bool = false;
	for page_el in html.select(".fed-page-info > a").array() {
		let page_node = match page_el.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		if page_node.text().read() == "尾页"
			&& !page_node
				.attr("onclick")
				.read()
				.contains(&format!("show('{}')", page.to_string()))
		{
			has_next = true;
			break;
		}
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: has_next,
	})
}

pub fn parse_manga_details(html: Node, manga_id: String) -> Result<Manga> {
	let title = html
		.select("dl.fed-deta-info h1.fed-part-eone")
		.text()
		.read();
	let author = html
		.select("dl.fed-deta-info ul.fed-part-rows li:nth-child(3) a")
		.text()
		.read();
	let desc = html
		.select("dl.fed-deta-info ul.fed-part-rows li:last-child div")
		.text()
		.read()
		.replace("简介", "");
	let image = html
		.select("dl.fed-deta-info a.fed-list-pics")
		.attr("data-original")
		.read();
	let url = format!("{}/manga-{}/", BASE_URL, manga_id);

	let manga = Manga {
		id: manga_id,
		cover: image,
		title,
		author: author.clone(),
		artist: author,
		description: desc,
		url,
		categories: vec![],
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll,
	};

	Ok(manga)
}

pub fn get_chapter_list(html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for li_ref in html.select(".all_data_list ul li").array() {
		let elem = match li_ref.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};

		let url = elem.select("a").attr("href").read();
		let parsed_url = url.replace(".html", "");
		let id_parts: Vec<&str> = parsed_url.split('/').collect();

		let id = format!("{}/{}", id_parts[2], id_parts[3]);
		let title = elem.select("a").attr("title").read();

		let volume = id_parts[1].parse::<f32>().unwrap_or(-1.0);
		let chapter = title
			.clone()
			.split(" ")
			.next()
			.unwrap()
			.replace(['第', '话'], "")
			.parse::<f32>()
			.unwrap_or(-1.0);

		let chapter = Chapter {
			id,
			title,
			volume,
			chapter,
			date_updated: -1.0,
			scanlator: String::new(),
			url,
			lang: String::from("zh"),
		};

		chapters.push(chapter);
	}

	Ok(chapters)
}

pub fn get_page_list(base_url: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	let res = Request::new(base_url.as_str(), HttpMethod::Get);
	let html = res.html()?;

	let base_data_re = Regex::new(r#"C_DATA\s*=\s*'(?<base_data>\S*)'"#).unwrap();
	let base_data_raw = html.select("script").to_string();
	let base_data_caps = base_data_re.captures(&base_data_raw).unwrap();
	let base_data = &base_data_caps["base_data"];
	println!("base_data: {}", base_data);

	let page_list = api::get_page_list(&base_data)?;

	for (index, url) in page_list.iter().enumerate() {
		println!("{} {}", index + 1, url);

		pages.push(Page {
			index: index as i32,
			url: url.to_string(),
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String) {
	url.push_str(BASE_URL);

	let mut is_searching = false;

	let mut search_string = String::new();
	let mut genre: &str = FILTER_GENRE[0];
	let mut progress: &str = FILTER_PROGRESS[0];
	let mut sort_by = SORT[0];

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.as_string() {
					search_string
						.push_str(encode_uri(&filter_value.read().to_lowercase()).as_str());
					is_searching = true;
				}
			}
			FilterType::Select => {
				let index = filter.value.as_int().unwrap_or(0) as usize;
				match filter.name.as_str() {
					"剧情" => genre = FILTER_GENRE[index],
					"进度" => progress = FILTER_PROGRESS[index],
					_ => continue,
				};
			}
			FilterType::Sort => {
				let Ok(obj) = filter.value.as_object() else {
					continue;
				};
				let index = obj.get("index").as_int().unwrap_or(0) as usize;
				sort_by = SORT[index];
			}
			_ => continue,
		}
	}

	if is_searching {
		let search_page_str = format!(
			"/search?type=1&searchString={}&page={}",
			search_string,
			page.to_string()
		);
		url.push_str(search_page_str.as_str());
	} else {
		let mut filter_values: Vec<String> = Vec::new();

		filter_values.push(format!("orderBy={}", sort_by));
		filter_values.push(format!("page={}", page.to_string()));

		if !genre.is_empty() && genre != "all" {
			filter_values.push(format!("mainCategoryId={}", genre));
		}

		if !progress.is_empty() && progress != "all" {
			filter_values.push(format!("status={}", progress));
		}

		let filter_str = filter_values.join("&");
		let page_str = format!("/show?{}", filter_str);

		url.push_str(page_str.as_str())
	}
}