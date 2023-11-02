//! Types for representing RPSL components.
use std::{ops::Index, option::Option};

/// Represents a RPSL attribute.
#[derive(Debug, PartialEq, Eq)]
pub struct Attribute {
    /// The name of the attribute.
    pub name: String,
    /// The values of the attribute.
    /// Single line attributes only have one value, while for multi-line attributes every line is a value.
    /// An empty value or one containing only whitespace is represented as None.
    pub value: AttributeValue,
}

impl Attribute {
    /// Create a new RPSL attribute from a name and an attribute value.
    #[must_use]
    pub fn new(name: String, value: AttributeValue) -> Self {
        Attribute { name, value }
    }
}

impl From<(&str, &str)> for Attribute {
    /// Examples
    /// ```
    /// # use rpsl_parser::rpsl;
    /// assert_eq!(
    ///     rpsl::Attribute::from(("role", "ACME Company")),
    ///     rpsl::Attribute::new(
    ///         "role".to_string(),
    ///         rpsl::AttributeValue::SingleLine(Some("ACME Company".to_string()))
    ///     )
    /// );
    /// ```
    fn from(name_value: (&str, &str)) -> Self {
        let (name, value) = name_value;
        Attribute::new(name.to_string(), value.into())
    }
}

impl From<(&str, Vec<&str>)> for Attribute {
    /// Examples
    /// ```
    /// # use rpsl_parser::rpsl;
    /// assert_eq!(
    ///     rpsl::Attribute::from((
    ///        "address",
    ///         vec!["Packet Street 6", "128 Series of Tubes", "Internet"],
    ///     )),
    ///     rpsl::Attribute::new(
    ///         "address".to_string(),
    ///         rpsl::AttributeValue::MultiLine(vec![
    ///             Some("Packet Street 6".to_string()),
    ///             Some("128 Series of Tubes".to_string()),
    ///             Some("Internet".to_string())
    ///         ])
    ///     )
    /// );
    /// ```
    fn from(name_values: (&str, Vec<&str>)) -> Self {
        let (name, values) = name_values;
        Attribute::new(name.to_string(), values.into())
    }
}

/// The value of a RPSL attribute.
#[derive(Debug, PartialEq, Eq)]
pub enum AttributeValue {
    SingleLine(Option<String>),
    MultiLine(Vec<Option<String>>),
}

impl From<&str> for AttributeValue {
    /// Examples
    /// ```
    /// # use rpsl_parser::rpsl;
    ///
    /// assert_eq!(
    ///     rpsl::AttributeValue::from("ACME Company"),
    ///     rpsl::AttributeValue::SingleLine(Some("ACME Company".to_string()))
    /// );
    /// assert_eq!(rpsl::AttributeValue::from(""), rpsl::AttributeValue::SingleLine(None));
    /// assert_eq!(rpsl::AttributeValue::from("   "), rpsl::AttributeValue::SingleLine(None));
    /// ```
    fn from(value: &str) -> Self {
        AttributeValue::SingleLine({
            if value.trim().is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        })
    }
}

impl From<Vec<&str>> for AttributeValue {
    /// Examples
    /// ```
    /// # use rpsl_parser::rpsl;
    /// assert_eq!(
    ///     rpsl::AttributeValue::from(vec![
    ///         "Packet Street 6",
    ///         "128 Series of Tubes",
    ///         "Internet"
    ///     ]),
    ///     rpsl::AttributeValue::MultiLine(vec![
    ///         Some("Packet Street 6".to_string()),
    ///         Some("128 Series of Tubes".to_string()),
    ///         Some("Internet".to_string())
    ///     ])
    /// );
    /// assert_eq!(
    ///     rpsl::AttributeValue::from(vec!["Packet Street 6"]),
    ///     rpsl::AttributeValue::SingleLine(Some("Packet Street 6".to_string())),
    /// );
    /// ```
    fn from(values: Vec<&str>) -> Self {
        if values.len() == 1 {
            return values[0].into();
        }

        AttributeValue::MultiLine(
            values
                .iter()
                .map(|v| {
                    if v.trim().is_empty() {
                        return None;
                    }
                    Some((*v).to_string())
                })
                .collect(),
        )
    }
}

/// Represents a RPSL object.
///
/// # Examples
///
/// Create a new role object
/// ```
/// # use rpsl_parser::rpsl;
/// let role_acme = rpsl::Object::new(vec![
///     rpsl::Attribute::new("role".to_string(), vec![Some("ACME Company".to_string())]),
///     rpsl::Attribute::new(
///         "address".to_string(),
///         vec![Some("Packet Street 6".to_string())],
///     ),
///     rpsl::Attribute::new(
///         "address".to_string(),
///         vec![Some("128 Series of Tubes".to_string())],
///     ),
///     rpsl::Attribute::new("address".to_string(), vec![Some("Internet".to_string())]),
///     rpsl::Attribute::new(
///         "email".to_string(),
///         vec![Some("rpsl-parser@github.com".to_string())],
///     ),
///     rpsl::Attribute::new("nic-hdl".to_string(), vec![Some("RPSL1-RIPE".to_string())]),
///     rpsl::Attribute::new("source".to_string(), vec![Some("RIPE".to_string())]),
/// ]);
/// ```
///
/// And then print the `email` attribute by it's index
/// ```
/// # use rpsl_parser::rpsl;
/// # let role_acme = rpsl::Object::new(vec![
/// #     rpsl::Attribute::new("role".to_string(), vec![Some("ACME Company".to_string())]),
/// #     rpsl::Attribute::new(
/// #         "address".to_string(),
/// #         vec![Some("Packet Street 6".to_string())],
/// #     ),
/// #     rpsl::Attribute::new(
/// #         "address".to_string(),
/// #         vec![Some("128 Series of Tubes".to_string())],
/// #     ),
/// #     rpsl::Attribute::new("address".to_string(), vec![Some("Internet".to_string())]),
/// #     rpsl::Attribute::new(
/// #         "email".to_string(),
/// #         vec![Some("rpsl-parser@github.com".to_string())],
/// #     ),
/// #     rpsl::Attribute::new("nic-hdl".to_string(), vec![Some("RPSL1-RIPE".to_string())]),
/// #     rpsl::Attribute::new("source".to_string(), vec![Some("RIPE".to_string())]),
/// # ]);
/// println!("{:#?}", role_acme[4]);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Object(Vec<Attribute>);

impl Object {
    /// Create a new RPSL object from a vector of attributes.
    #[must_use]
    pub fn new(attributes: Vec<Attribute>) -> Self {
        Object(attributes)
    }
}

impl From<Vec<(&str, &str)>> for Object {
    /// Examples
    /// ```
    /// # use rpsl_parser::rpsl;
    /// assert_eq!(
    ///     rpsl::Object::from(vec![
    ///         ("role", "ACME Company"),
    ///         ("address", "Packet Street 6"),
    ///         ("address", "128 Series of Tubes"),
    ///         ("address", "Internet"),
    ///     ]),
    ///     rpsl::Object::new(vec![
    ///         rpsl::Attribute::from(("role", "ACME Company")),
    ///         rpsl::Attribute::from(("address", "Packet Street 6")),
    ///         rpsl::Attribute::from(("address", "128 Series of Tubes")),
    ///         rpsl::Attribute::from(("address", "Internet")),
    ///     ])
    /// );
    /// ```
    fn from(attributes: Vec<(&str, &str)>) -> Self {
        let attributes: Vec<Attribute> = attributes.iter().map(|a| (*a).into()).collect();
        Object::new(attributes)
    }
}

impl Index<usize> for Object {
    type Output = Attribute;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IntoIterator for Object {
    type Item = Attribute;
    type IntoIter = std::vec::IntoIter<Attribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

///Represents a collection of RPSL objects, for example as the result of a whois query.
#[derive(Debug, PartialEq, Eq)]
pub struct ObjectCollection(Vec<Object>);

impl ObjectCollection {
    /// Create a new RPSL object collection from a vector of objects.
    #[must_use]
    pub fn new(objects: Vec<Object>) -> Self {
        ObjectCollection(objects)
    }
}

impl Index<usize> for ObjectCollection {
    type Output = Object;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IntoIterator for ObjectCollection {
    type Item = Object;
    type IntoIter = std::vec::IntoIter<Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Create an RPSL object collection from a vector of slices parsed from RPSL text.
impl From<Vec<Vec<Attribute>>> for ObjectCollection {
    fn from(object_slices: Vec<Vec<Attribute>>) -> Self {
        let mut objects: Vec<Object> = Vec::with_capacity(object_slices.len());

        for object in object_slices {
            objects.push(Object::new(object));
        }

        ObjectCollection(objects)
    }
}
