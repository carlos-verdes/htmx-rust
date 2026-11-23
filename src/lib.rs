#![cfg_attr(feature = "maud", doc = include_str!("../README.md"))]

mod html;
mod view;

pub use html::Html;
pub use view::{Layout, Outlet, View, Views};
