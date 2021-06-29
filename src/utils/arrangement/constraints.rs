use std::convert::TryInto;

use super::helper::*;
use super::{ColumnDisplayInfo, DisplayInfos};
use crate::style::{Boundary, ColumnConstraint, ColumnConstraint::*};
use crate::{Column, Table};

/// Look at given constraints of a column and check if some of them can be resolved at the very
/// beginning.
///
/// For example:
/// - We get an absolute width.
/// - MinWidth constraints on columns, whose content is garantueed to be smaller than the specified
///     minimal width.
/// - The Column is supposed to be hidden.
pub fn evaluate(
    table: &Table,
    column: &Column,
    infos: &mut DisplayInfos,
    table_width: Option<usize>,
    visible_columns: usize,
) {
    match &column.constraint {
        Some(ContentWidth) => {
            let info = ColumnDisplayInfo::new(column, column.max_content_width);
            infos.insert(column.index, info);
        }
        Some(Absolute(boundary)) => {
            if let Some(width) =
                absolute_value_from_boundary(table, boundary, visible_columns, table_width)
            {
                // The column should get always get a fixed width.
                let width = absolute_width_with_padding(column, width);
                let info = ColumnDisplayInfo::new(column, width);
                infos.insert(column.index, info);
            }
        }
        Some(Hidden) => {
            let mut info = ColumnDisplayInfo::new(column, column.max_content_width);
            info.is_hidden = true;
            infos.insert(column.index, info);
        }
        _ => {}
    }

    if let Some(min_width) =
        get_min_constraint(table, &column.constraint, table_width, visible_columns)
    {
        // In case a min_width is specified, we may already fix the size of the column.
        // We do this, if we know that the content is smaller than the min size.
        if column.get_max_width() <= min_width {
            let width = absolute_width_with_padding(column, min_width);
            let info = ColumnDisplayInfo::new(column, width);
            infos.insert(column.index, info);
        }
    }
}

/// A little wrapper, which resolves possible lower boundary constraints to their actual value for
/// the current table and terminal width.
///
/// This returns the value of absolute characters that are allowed to be in this column. \
/// Lower boundaries with [Boundary::Fixed] just return their internal value. \
/// Lower boundaries with [Boundary::Percentage] return the percental amount of the current table
/// width.
pub fn get_min_constraint(
    table: &Table,
    constraint: &Option<ColumnConstraint>,
    table_width: Option<usize>,
    visible_columns: usize,
) -> Option<u16> {
    let constraint = if let Some(constraint) = constraint {
        constraint
    } else {
        return None;
    };

    match constraint {
        LowerBoundary(boundary)
        | Boundaries {
            lower: boundary, ..
        } => absolute_value_from_boundary(table, boundary, visible_columns, table_width),
        _ => None,
    }
}

/// A little wrapper, which resolves possible upper boundary constraints to their actual value for
/// the current table and terminal width.
///
/// This returns the value of absolute characters that are allowed to be in this column. \
/// Upper boundaries with [Boundary::Fixed] just return their internal value. \
/// Upper boundaries with [Boundary::Percentage] return the percental amount of the current table
/// width.
pub fn get_max_constraint(
    table: &Table,
    constraint: &Option<ColumnConstraint>,
    table_width: Option<usize>,
    visible_columns: usize,
) -> Option<u16> {
    let constraint = if let Some(constraint) = constraint {
        constraint
    } else {
        return None;
    };

    match constraint {
        UpperBoundary(boundary)
        | Boundaries {
            upper: boundary, ..
        } => absolute_value_from_boundary(table, boundary, visible_columns, table_width),
        _ => None,
    }
}

/// Resolve an absolute value from a given boundary
pub fn absolute_value_from_boundary(
    table: &Table,
    boundary: &Boundary,
    visible_columns: usize,
    table_width: Option<usize>,
) -> Option<u16> {
    match boundary {
        Boundary::Fixed(width) => Some(*width),
        Boundary::Percentage(percent) => {
            // Don't return a value, if we cannot determine the current table width.
            let table_width = table_width?;

            // Enforce at most 100%
            let percent = std::cmp::min(*percent, 100u16);

            // Subtract the borders from the table width.
            let width = table_width.saturating_sub(count_border_columns(table, visible_columns));

            // Calculate the absolute value in actual columns.
            let width = (width * usize::from(percent) / 100)
                .try_into()
                .unwrap_or(u16::MAX);
            Some(width)
        }
    }
}
