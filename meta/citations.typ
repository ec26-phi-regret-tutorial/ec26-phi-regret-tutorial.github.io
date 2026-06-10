#let citation_suffixes = (
  "a",
  "b",
  "c",
  "d",
  "e",
  "f",
  "g",
  "h",
  "i",
  "j",
  "k",
  "l",
  "m",
  "n",
  "o",
  "p",
  "q",
  "r",
  "s",
  "t",
  "u",
  "v",
  "w",
  "x",
  "y",
  "z",
)

#let citation_label_overrides = (
  ("Dagan24:From", "DDFS24"),
)

#let citation_key_name(key) = {
  let raw = str(key)
  if raw.starts-with("<") and raw.ends-with(">") {
    raw.slice(1, -1)
  } else {
    raw
  }
}

#let citation_bib_source = read("refs.bib")

#let citation_bib_entry(key_name) = {
  let marker = "{" + key_name + ","
  let found = ""
  for chunk in citation_bib_source.split("@") {
    if chunk.contains(marker) {
      found = chunk
    }
  }
  found
}

#let citation_braced_value(raw) = {
  let value = raw.trim()
  if value.starts-with("{") {
    let depth = 0
    let out = ""
    for ch in value {
      if ch == "{" {
        if depth > 0 {
          out += ch
        }
        depth += 1
      } else if ch == "}" {
        depth -= 1
        if depth == 0 {
          break
        }
        out += ch
      } else if depth > 0 {
        out += ch
      }
    }
    out
  } else if value.starts-with("\"") {
    let out = ""
    for ch in value.slice(1) {
      if ch == "\"" {
        break
      }
      out += ch
    }
    out
  } else {
    value.split(",").first().trim()
  }
}

#let citation_bib_field(key_name, field) = {
  let entry = citation_bib_entry(key_name)
  let parts = entry.split(regex("(?i)" + field + "\\s*="))
  if parts.len() < 2 {
    ""
  } else {
    citation_braced_value(parts.at(1))
  }
}

#let citation_clean_name(name) = {
  name
    .replace(regex("[{}]"), "")
    .replace(regex("\\\\[a-zA-Z]+"), "")
    .replace(regex("\\\\."), "")
    .replace(regex("\\s+"), " ")
    .trim()
}

#let citation_author_last_names(key_name) = {
  let author = citation_bib_field(key_name, "author")
  if author == "" {
    ()
  } else {
    author
      .split(regex("\\s+and\\s+"))
      .map(name => {
        let clean = citation_clean_name(name)
        if clean.contains(",") {
          clean.split(",").first().trim()
        } else {
          clean.split(regex("\\s+")).last().trim()
        }
      })
  }
}

#let citation_lowercase = "abcdefghijklmnopqrstuvwxyz"
#let citation_uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
#let citation_digits = "0123456789"

#let citation_is_ascii_letter(ch) = {
  ch in citation_lowercase or ch in citation_uppercase
}

#let citation_upper_char(ch) = {
  let out = ch
  for i in range(26) {
    if ch == citation_lowercase.at(i) {
      out = citation_uppercase.at(i)
    }
  }
  out
}

#let citation_lower_char(ch) = {
  let out = ch
  for i in range(26) {
    if ch == citation_uppercase.at(i) {
      out = citation_lowercase.at(i)
    }
  }
  out
}

#let citation_initial(name) = {
  let out = ""
  for ch in citation_clean_name(name) {
    if citation_is_ascii_letter(ch) {
      out = citation_upper_char(ch)
      break
    }
  }
  out
}

#let citation_alpha_prefix(text, count: 3) = {
  let out = ""
  for ch in citation_clean_name(text) {
    if citation_is_ascii_letter(ch) and out.len() < count {
      if out.len() == 0 {
        out += citation_upper_char(ch)
      } else {
        out += citation_lower_char(ch)
      }
    }
  }
  out
}

#let citation_year_suffix(key_name) = {
  let out = ""
  for ch in citation_bib_field(key_name, "year") {
    if ch in citation_digits {
      out += ch
    }
  }
  if out.len() >= 2 {
    out.slice(-2)
  } else {
    out
  }
}

#let citation_alpha_label_base(key_name) = {
  let override = none
  for (override_key, override_label) in citation_label_overrides {
    if override_key == key_name {
      override = override_label
    }
  }
  if override != none {
    override
  } else {
    let authors = citation_author_last_names(key_name)
    let prefix = if authors.len() == 0 {
      citation_alpha_prefix(key_name)
    } else if authors.len() == 1 {
      citation_alpha_prefix(authors.first())
    } else if authors.len() <= 4 {
      authors.map(citation_initial).join("")
    } else {
      authors.slice(0, 3).map(citation_initial).join("") + "+"
    }
    prefix + citation_year_suffix(key_name)
  }
}

#let citation_label(key, cited_keys: ()) = {
  let key_name = citation_key_name(key)
  let base = citation_alpha_label_base(key_name)
  let cited_key_names = cited_keys.map(citation_key_name)
  let group = cited_key_names.filter(group_key => citation_alpha_label_base(group_key) == base)
  let suffix = if group.len() > 1 {
    let suffix_index = 0
    for (i, group_key) in group.enumerate() {
      if group_key == key_name {
        suffix_index = i
      }
    }
    citation_suffixes.at(suffix_index)
  } else {
    ""
  }
  base + suffix
}

#let citation_label_text(key, cited_keys: (), ..supplement) = {
  let suffix = if supplement.pos().len() > 0 {
    ", " + str(supplement.pos().first())
  } else {
    let named = supplement.named()
    if "supplement" in named {
      ", " + str(named.at("supplement"))
    } else {
      ""
    }
  }
  citation_label(key, cited_keys: cited_keys) + suffix
}

#let citation_bracket(key, cited_keys: (), ..supplement) = {
  "[" + citation_label_text(key, cited_keys: cited_keys, ..supplement) + "]"
}

#let citation_author_text(key) = {
  let names = citation_author_last_names(citation_key_name(key))
  if names.len() == 0 {
    citation_key_name(key)
  } else if names.len() == 1 {
    names.first()
  } else if names.len() == 2 {
    names.first() + " and " + names.last()
  } else {
    names.slice(0, -1).join(", ") + " and " + names.last()
  }
}

#let citation_html_id(key) = {
  "bib-" + citation_key_name(key).replace(regex("[^A-Za-z0-9_-]+"), "-")
}
