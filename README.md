<h1 align="center"><code>rpsl-parser</code></h1>

<div align="center">
  <a href="https://github.com/srv6d/rpsl-parser/actions">
    <img src="https://github.com/srv6d/rpsl-parser/workflows/CI/badge.svg" alt="CI status">
  </a>
  <a href="https://codspeed.io/SRv6d/rpsl-parser">
    <img src="https://img.shields.io/endpoint?url=https://codspeed.io/badge.json" alt="CodSpeed Badge">
  </a>
  <a href="https://crates.io/crates/rpsl-parser">
    <img src="https://img.shields.io/crates/v/rpsl-parser.svg" alt="Cargo version">
  </a>
  
</div>
<br>

An [RFC 2622] conformant Routing Policy Specification Language (RPSL) parser with a focus on speed and correctness.

⚡️ 130-250x faster than other parsers\
📰 Complete implementation for multiline RPSL values\
💬 Able to parse objects directly from whois server responses\
🧠 Low memory footprint by leveraging zero-copy\
🧪 Robust parsing of any valid input ensured by Property Based Tests

[**Docs**](https://docs.rs/rpsl-parser/latest/rpsl/) | [**Performance**](https://github.com/SRv6d/rpsl-parser/tree/main/docs/benchmark)

## Usage

### Parsing RPSL objects

A string containing an object in RPSL notation can be parsed to an [ObjectView] using the [parse_object] function.

```rust,ignore
use rpsl::parse_object;

let role_acme = "
role:        ACME Company
address:     Packet Street 6
address:     128 Series of Tubes
address:     Internet
email:       rpsl-parser@github.com
nic-hdl:     RPSL1-RIPE
source:      RIPE

";
let parsed = parse_object(role_acme)?;
```

The returned [ObjectView] allows access to the attributes contained within in form of [AttributeView]s, a type that contains references to the data it represents, making the parser very memory efficient and performant, since no allocation is needed during parsing.

```ignore
role:           ACME Company ◀─────────────── &"role"    ───  &"ACME Company"
address:        Packet Street 6 ◀──────────── &"address" ───  &"Packet Street 6"
address:        128 Series of Tubes ◀──────── &"address" ───  &"128 Series of Tubes"
address:        Internet ◀─────────────────── &"address" ───  &"Internet"
email:          rpsl-parser@github.com ◀───── &"email"   ───  &"rpsl-parser@github.com"
nic-hdl:        RPSL1-RIPE ◀───────────────── &"nic-hdl" ───  &"RPSL1-RIPE"
source:         RIPE ◀─────────────────────── &"source"  ───  &"RIPE"
```

```rust,ignore
println!("{:#?}", parsed);

ObjectView(
    [
        AttributeView {
            name: NameView("role",),
            value: SingleLine(Some("ACME Company")),
        },
        AttributeView {
            name: NameView("address"),
            value: SingleLine(Some("Packet Street 6")),
        },
        AttributeView {
            name: NameView("address"),
            value: SingleLine(Some("128 Series of Tubes")),
        },
        AttributeView {
            name: NameView("address"),
            value: SingleLine(Some("Internet")),
        },
        AttributeView {
            name: NameView("email"),
            value: SingleLine(Some("rpsl-parser@github.com")),
        },
        AttributeView {
            name: NameView("nic-hdl"),
            value: SingleLine(Some("RPSL1-RIPE")),
        },
        AttributeView {
            name: NameView("source"),
            value: SingleLine(Some("RIPE")),
        },
    ]
)
```

Each [AttributeView] can be accessed by its index and has a name and optional value(s).

```rust,ignore
println!("{:#?}", parsed[1]);

AttributeView {
    name: NameView("address"),
    value: SingleLine(Some("Packet Street 6")),
}
```

Since RPSL attribute values can either be single- or multiline, two different types are used to represent them. See [Attribute] and [parse_object] for more details and examples.

[ObjectView]s can be converted to as well as compared directly with their owned counterpart [Object].

```rust,ignore
let owned = object! {
    "role":        "ACME Company";
    "address":     "Packet Street 6";
    "address":     "128 Series of Tubes";
    "address":     "Internet";
    "email":       "rpsl-parser@github.com";
    "nic-hdl":     "RPSL1-RIPE";
    "source":      "RIPE";
};

assert_eq!(role_acme, owned);
assert_eq!(role_acme.to_owned(), owned);
```

### Parsing a WHOIS server response

WHOIS servers often respond to queries by returning multiple related objects.
An example ARIN query for `AS32934` will return with the requested `ASNumber` object first, followed by its associated `OrgName`:

```sh
$ whois -h whois.arin.net AS32934
ASNumber:       32934
ASName:         FACEBOOK
ASHandle:       AS32934
RegDate:        2004-08-24
Updated:        2012-02-24
Comment:        Please send abuse reports to abuse@facebook.com
Ref:            https://rdap.arin.net/registry/autnum/32934


OrgName:        Facebook, Inc.
OrgId:          THEFA-3
Address:        1601 Willow Rd.
City:           Menlo Park
StateProv:      CA
PostalCode:     94025
Country:        US
RegDate:        2004-08-11
Updated:        2012-04-17
Ref:            https://rdap.arin.net/registry/entity/THEFA-3


```

To extract each individual object, the [parse_whois_response] function can be used to parse the response into a `Vec` containing all individual [ObjectView]s within the response. Examples can be found in the function documentation.

[RFC 2622]: https://datatracker.ietf.org/doc/html/rfc2622
[Object]: https://docs.rs/rpsl-parser/latest/rpsl/struct.Object.html
[ObjectView]: https://docs.rs/rpsl-parser/latest/rpsl/struct.ObjectView.html
[Attribute]: https://docs.rs/rpsl-parser/latest/rpsl/struct.Attribute.html
[AttributeView]: https://docs.rs/rpsl-parser/latest/rpsl/struct.AttributeView.html
[parse_object]: https://docs.rs/rpsl-parser/latest/rpsl/fn.parse_object.html
[parse_whois_response]: https://docs.rs/rpsl-parser/latest/rpsl/fn.parse_whois_response.html
