#[grammar(
	file = "grammar.abnf",
	export("IRI", "IRI-reference" as IriRef, "iauthority" as Authority, "ihost" as Host, "iuserinfo" as UserInfo, "ipath" as Path, "isegment" as Segment, "iquery" as Query, "ifragment" as Fragment)
)]
pub(crate) mod grammar {}

include!(concat!(env!("OUT_DIR"), "/iri.rs"));
