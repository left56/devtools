/// Fixed tool groups. Add a variant here before inventing a new sidebar section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Data,
    Encoding,
    Text,
    Time,
    Generators,
    Web,
    Security,
}

impl Category {
    pub const ALL: [Self; 7] = [
        Self::Data,
        Self::Encoding,
        Self::Text,
        Self::Time,
        Self::Generators,
        Self::Web,
        Self::Security,
    ];

    /// Stable English id. Not a display string.
    pub fn id(self) -> &'static str {
        match self {
            Self::Data => "data",
            Self::Encoding => "encoding",
            Self::Text => "text",
            Self::Time => "time",
            Self::Generators => "generators",
            Self::Web => "web",
            Self::Security => "security",
        }
    }

    /// Chinese label used by the shell. Tool names themselves stay in the
    /// English form developers search for ("JSON", "JWT").
    pub fn label(self) -> &'static str {
        match self {
            Self::Data => "数据",
            Self::Encoding => "编码",
            Self::Text => "文本",
            Self::Time => "时间",
            Self::Generators => "生成",
            Self::Web => "网络",
            Self::Security => "安全",
        }
    }
}
