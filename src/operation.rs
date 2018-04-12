#[derive(PartialEq, Debug)]
pub enum Operation
{
    Exit,
    List,
    Get(u8)
}

#[derive(Debug)]
pub enum OperationErrorKind
{
    InvalidOperation(u8),
    NullArgument(u8)
}

#[derive(Debug)]
pub struct OperationError
{
    kind: OperationErrorKind
}

impl OperationError
{
    pub fn new(kind: OperationErrorKind) -> OperationError
    {
        OperationError // return new struct
        {
            kind: kind
        }
    }
}

impl ::std::error::Error for OperationError
{
    fn description(&self) -> &str
    {
        "Operation error"
    }
}

impl ::std::fmt::Display for OperationError
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
    {
        write!(f, "Operation error: {:?}", self.kind)
    }
}

impl Operation
{
    pub fn from(operation: u8, argument: Option<u8>) -> Result<Operation, OperationError>
    {
        match operation
        {
            // 0 => terminator
            1 => Ok(Operation::Exit),
            2 => Ok(Operation::List),
            3 => match argument
            {
                Some(arg) => Ok(Operation::Get(arg)),
                None => Err(OperationError::new(OperationErrorKind::NullArgument(operation)))
            }
            _ => Err(OperationError::new(OperationErrorKind::InvalidOperation(operation)))
        }
    }
}
