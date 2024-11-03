//! Módulo para operaciones de tablas.

use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
};

use crate::protocol::{aliases::results::Result, errors::error::Error};

use super::table_path::TablePath;

