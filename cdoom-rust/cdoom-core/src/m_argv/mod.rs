pub const MAXARGVS: usize = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResponseParseError {
    UnclosedQuote,
    TooManyArguments,
}

fn ascii_eq_ignore_case(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(&a, &b)| a.eq_ignore_ascii_case(&b))
}

pub fn check_parm_with_args(args: &[Option<&[u8]>], check: &[u8], num_args: usize) -> usize {
    if args.len() <= num_args {
        return 0;
    }

    let limit = args.len() - num_args;
    let mut index = 1;

    while index < limit {
        let Some(arg) = args[index] else {
            break;
        };

        if ascii_eq_ignore_case(check, arg) {
            return index;
        }

        index += 1;
    }

    0
}

pub fn parse_response_file(bytes: &[u8]) -> Result<Vec<Vec<u8>>, ResponseParseError> {
    let mut args = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        let start;

        if bytes[index] == b'"' {
            index += 1;
            start = index;

            while index < bytes.len() && bytes[index] != b'"' && bytes[index] != b'\n' {
                index += 1;
            }

            if index >= bytes.len() || bytes[index] == b'\n' {
                return Err(ResponseParseError::UnclosedQuote);
            }

            push_response_arg(&mut args, &bytes[start..index])?;
            index += 1;
        } else {
            start = index;

            while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
                index += 1;
            }

            push_response_arg(&mut args, &bytes[start..index])?;

            if index < bytes.len() {
                index += 1;
            }
        }
    }

    Ok(args)
}

fn push_response_arg(args: &mut Vec<Vec<u8>>, arg: &[u8]) -> Result<(), ResponseParseError> {
    if args.len() >= MAXARGVS {
        return Err(ResponseParseError::TooManyArguments);
    }

    args.push(arg.to_vec());
    Ok(())
}
