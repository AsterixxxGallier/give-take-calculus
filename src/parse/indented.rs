use crate::parse::{skip_empty_lines, ParseError, SourceLocationLines};

pub(super) fn parse_indented<'s, R: 's>(
    location: SourceLocationLines<'s>,
    parse: impl FnOnce(SourceLocationLines<'s>) -> Result<(SourceLocationLines<'s>, R), ParseError<'s>>,
    empty: impl FnOnce() -> R,
) -> Result<(SourceLocationLines<'s>, R), ParseError<'s>> {
    if let Some(location) = skip_empty_lines(location) {
        let line = location.first().expect("should not be empty");

        let indentation = line.take_while_whitespace();
        if let Some(reference_indentation) = location.reference_indentation {
            if indentation.starts_with(reference_indentation.as_str())
                && indentation.as_str() != reference_indentation.as_str()
            {
                let old_reference_indentation = location.reference_indentation;
                let (mut location, context) =
                    parse(location.with_reference_indentation(indentation))?;
                location.reference_indentation = old_reference_indentation;
                Ok((location, context))
            } else {
                // indentation has been reduced
                //   => assume the next line is outside the context (one indentation level less)
                //   => the indented context is empty
                // or stayed the same
                //   => the next line belongs to the current context
                //   => the indented context is empty
                Ok((location, empty()))
            }
        } else {
            if !indentation.is_empty() {
                let old_reference_indentation = location.reference_indentation;
                let (mut location, context) =
                    parse(location.with_reference_indentation(indentation))?;
                location.reference_indentation = old_reference_indentation;
                Ok((location, context))
            } else {
                // indentation stayed the same
                Ok((location, empty()))
            }
        }
    } else {
        Ok((location, empty()))
    }
}

pub(super) fn parse_with_indentation<'s, E: 's>(
    mut location: SourceLocationLines<'s>,
    parse_element: impl Fn(
        SourceLocationLines<'s>,
    ) -> Result<(SourceLocationLines<'s>, E), ParseError<'s>>,
) -> Result<(SourceLocationLines<'s>, Vec<E>), ParseError<'s>> {
    let mut elements = Vec::new();

    while let Some(new_location) = skip_empty_lines(location) {
        location = new_location;

        // skip_empty_lines shouldn't return Some(new_location) with empty new_location
        let line = new_location.first().expect("should not be empty");

        let indentation = line.take_while_whitespace();
        if let Some(reference_indentation) = location.reference_indentation {
            if !line.starts_with(reference_indentation.as_str()) {
                // indentation has been reduced
                // => assume the next line is outside the context (one indentation level less)
                break;
            }
            if indentation.as_str() != reference_indentation.as_str() {
                // indentation has changed in another way
                return Err(ParseError::IndentationMismatch {
                    expected_indentation: reference_indentation,
                    actual_indentation: indentation,
                });
            }
        } else {
            if !indentation.is_empty() {
                return Err(ParseError::UnexpectedIndentation { indentation });
            }
        }

        let (new_location, element) = parse_element(location)?;
        location = new_location;
        elements.push(element);
    }

    Ok((location, elements))
}
