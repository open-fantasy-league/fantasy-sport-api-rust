use std::fmt;

#[derive(Debug, Clone)]
pub struct InvalidTeamError {
    pub description: String,
}

impl fmt::Display for InvalidTeamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid team: {}", self.description)
    }
}

impl std::error::Error for InvalidTeamError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug, Clone)]
pub struct InvalidInputError<'a> {
    pub description: &'a str,
}

impl<'a> fmt::Display for InvalidInputError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid input: {}", self.description)
    }
}

impl<'a> std::error::Error for InvalidInputError<'a> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug, Clone)]
pub struct CustomError<'a> {
    pub description: &'a str,
}

impl<'a> fmt::Display for CustomError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.description)
    }
}

impl<'a> std::error::Error for CustomError<'a> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
