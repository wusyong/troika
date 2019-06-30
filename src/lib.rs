/*
 * Copyright (C) 2019 Yu-Wei Wu
 * All Rights Reserved.
 * This is free software; you can redistribute it and/or modify it under the
 * terms of the MIT license. A copy of the license can be found in the file
 * "LICENSE" at the root of this distribution.
 */

extern crate failure;

mod macros;
pub mod troika;
pub mod ftroika;

use std::result;

pub type Result<T> = result::Result<T, failure::Error>;
