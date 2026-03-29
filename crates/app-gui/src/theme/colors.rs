#![allow(dead_code)]
use egui::Color32;

pub const SURFACE: Color32 = Color32::from_rgb(0x0b, 0x13, 0x26);
pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(0x17, 0x1f, 0x33);
pub const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(0x13, 0x1b, 0x2e);
pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(0x22, 0x2a, 0x3d);
pub const SURFACE_CONTAINER_HIGHEST: Color32 = Color32::from_rgb(0x2d, 0x34, 0x49);
pub const SURFACE_CONTAINER_LOWEST: Color32 = Color32::from_rgb(0x06, 0x0e, 0x20);

pub const ON_SURFACE: Color32 = Color32::from_rgb(0xda, 0xe2, 0xfd);
pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(0xc6, 0xc6, 0xcd);

pub const PRIMARY: Color32 = Color32::from_rgb(0xad, 0xc6, 0xff);
pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0x00, 0x16, 0x3a);
pub const ON_PRIMARY: Color32 = Color32::from_rgb(0x00, 0x2e, 0x6a);
pub const ON_PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0x35, 0x7d, 0xf1);

pub const SECONDARY: Color32 = Color32::from_rgb(0xff, 0xb6, 0x90);
pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(0xec, 0x6a, 0x06);
pub const ON_SECONDARY: Color32 = Color32::from_rgb(0x55, 0x21, 0x00);

pub const OUTLINE: Color32 = Color32::from_rgb(0x90, 0x90, 0x97);
pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(0x45, 0x46, 0x4d);

pub const ERROR: Color32 = Color32::from_rgb(0xff, 0xb4, 0xab);
pub const ON_ERROR: Color32 = Color32::from_rgb(0x69, 0x00, 0x05);

pub const SUCCESS_BADGE: Color32 = Color32::from_rgb(0x10, 0xb9, 0x81); // Emerald 500
pub const WARNING_BADGE: Color32 = Color32::from_rgb(0xf5, 0x9e, 0x0b); // Amber 500
pub const CRITICAL_BADGE: Color32 = Color32::from_rgb(0xf4, 0x3f, 0x5e); // Rose 500
