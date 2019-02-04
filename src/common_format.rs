// Copyright 2017-2018 Kisio Digital and/or its affiliates.
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see
// <http://www.gnu.org/licenses/>.

use crate::collection::*;
use crate::model::Collections;
use crate::objects::{self, Date, ExceptionType};
use crate::read_utils::FileHandler;
use crate::utils::*;
use crate::utils::{de_from_date_string, ser_from_naive_date};
use crate::Result;
use chrono::{self, Datelike};
use csv;
use derivative::Derivative;
use failure::{bail, ResultExt};
use log::info;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path;

#[derive(Serialize, Deserialize, Debug, Derivative, PartialEq, Eq, Hash, Clone, Copy)]
#[derivative(Default)]
pub enum Availability {
    #[derivative(Default)]
    #[serde(rename = "0")]
    InformationNotAvailable,
    #[serde(rename = "1")]
    Available,
    #[serde(rename = "2")]
    NotAvailable,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CalendarDate {
    pub service_id: String,
    #[serde(
        deserialize_with = "de_from_date_string",
        serialize_with = "ser_from_naive_date"
    )]
    pub date: Date,
    pub exception_type: ExceptionType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Calendar {
    #[serde(rename = "service_id")]
    id: String,
    #[serde(deserialize_with = "de_from_u8", serialize_with = "ser_from_bool")]
    monday: bool,
    #[serde(deserialize_with = "de_from_u8", serialize_with = "ser_from_bool")]
    tuesday: bool,
    #[serde(deserialize_with = "de_from_u8", serialize_with = "ser_from_bool")]
    wednesday: bool,
    #[serde(deserialize_with = "de_from_u8", serialize_with = "ser_from_bool")]
    thursday: bool,
    #[serde(deserialize_with = "de_from_u8", serialize_with = "ser_from_bool")]
    friday: bool,
    #[serde(deserialize_with = "de_from_u8", serialize_with = "ser_from_bool")]
    saturday: bool,
    #[serde(deserialize_with = "de_from_u8", serialize_with = "ser_from_bool")]
    sunday: bool,
    #[serde(
        deserialize_with = "de_from_date_string",
        serialize_with = "ser_from_naive_date"
    )]
    start_date: Date,
    #[serde(
        deserialize_with = "de_from_date_string",
        serialize_with = "ser_from_naive_date"
    )]
    end_date: Date,
}

impl Calendar {
    fn get_valid_days(&self) -> Vec<chrono::Weekday> {
        let mut valid_days: Vec<chrono::Weekday> = vec![];
        if self.monday {
            valid_days.push(chrono::Weekday::Mon);
        }
        if self.tuesday {
            valid_days.push(chrono::Weekday::Tue);
        }
        if self.wednesday {
            valid_days.push(chrono::Weekday::Wed);
        }
        if self.thursday {
            valid_days.push(chrono::Weekday::Thu);
        }
        if self.friday {
            valid_days.push(chrono::Weekday::Fri);
        }
        if self.saturday {
            valid_days.push(chrono::Weekday::Sat);
        }
        if self.sunday {
            valid_days.push(chrono::Weekday::Sun);
        }

        valid_days
    }

    fn get_valid_dates(&self) -> BTreeSet<Date> {
        let valid_days = self.get_valid_days();
        let duration = self.end_date - self.start_date;
        (0..=duration.num_days())
            .map(|i| self.start_date + chrono::Duration::days(i))
            .filter(|d| valid_days.contains(&d.weekday()))
            .collect()
    }
}

fn manage_calendar_dates<H>(
    calendars: &mut CollectionWithId<objects::Calendar>,
    file_handler: &mut H,
    calendar_exists: bool,
) -> Result<()>
where
    for<'a> &'a mut H: FileHandler,
{
    let file = "calendar_dates.txt";
    let (reader, path) = file_handler.get_file_if_exists(file)?;
    match reader {
        None => {
            info!("Skipping {}", file);
            if !calendar_exists {
                bail!("calendar_dates.txt or calendar.txt not found");
            }
        }
        Some(reader) => {
            info!("Reading {}", file);

            let mut rdr = csv::Reader::from_reader(reader);
            for calendar_date in rdr.deserialize() {
                let calendar_date: CalendarDate =
                    calendar_date.with_context(ctx_from_path!(path))?;

                let is_inserted =
                    calendars
                        .get_mut(&calendar_date.service_id)
                        .map(|mut calendar| match calendar_date.exception_type {
                            ExceptionType::Add => {
                                calendar.dates.insert(calendar_date.date);
                            }
                            ExceptionType::Remove => {
                                calendar.dates.remove(&calendar_date.date);
                            }
                        });
                is_inserted.unwrap_or_else(|| {
                    if calendar_date.exception_type == ExceptionType::Add {
                        let mut dates = BTreeSet::new();
                        dates.insert(calendar_date.date);
                        calendars
                            .push(objects::Calendar {
                                id: calendar_date.service_id,
                                dates,
                            })
                            .unwrap();
                    }
                });
            }
        }
    }
    Ok(())
}

pub fn manage_calendars<H>(file_handler: &mut H, collections: &mut Collections) -> Result<()>
where
    for<'a> &'a mut H: FileHandler,
{
    let mut calendars: Vec<objects::Calendar> = vec![];
    let calendar_exists = {
        let file = "calendar.txt";
        let (calendar_reader, path) = file_handler.get_file_if_exists(file)?;
        match calendar_reader {
            None => {
                info!("Skipping {}", file);
                false
            }
            Some(calendar_reader) => {
                info!("Reading {}", file);
                let mut rdr = csv::Reader::from_reader(calendar_reader);
                for calendar in rdr.deserialize() {
                    let calendar: Calendar = calendar.with_context(ctx_from_path!(path))?;
                    calendars.push(objects::Calendar {
                        id: calendar.id.clone(),
                        dates: calendar.get_valid_dates(),
                    });
                }
                collections.calendars = CollectionWithId::new(calendars)?;
                true
            }
        }
    };

    manage_calendar_dates(&mut collections.calendars, file_handler, calendar_exists)?;

    Ok(())
}

pub fn write_calendar_dates(
    path: &path::Path,
    calendars: &CollectionWithId<objects::Calendar>,
) -> Result<()> {
    info!("Writing calendar_dates.txt");
    let calendar_dates_path = path.join("calendar_dates.txt");
    let mut wtr = csv::Writer::from_path(&calendar_dates_path)
        .with_context(ctx_from_path!(calendar_dates_path))?;
    for c in calendars.values() {
        for d in &c.dates {
            wtr.serialize(CalendarDate {
                service_id: c.id.clone(),
                date: *d,
                exception_type: ExceptionType::Add,
            })
            .with_context(ctx_from_path!(calendar_dates_path))?;
        }
    }
    wtr.flush()
        .with_context(ctx_from_path!(calendar_dates_path))?;

    Ok(())
}
