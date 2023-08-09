use serde::{Deserialize, Serialize};
use time::*;

//#[cfg(not(test))]
//pub type MockableDateTime = not_mockable::MockableDateTime;
//#[cfg(test)]
pub type MockableDateTime = mockable::MockableDateTime;

#[cfg(not(test))]
mod not_mockable {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MockableDateTime;

    impl MockableDateTime {
        pub fn now_utc(&self) -> OffsetDateTime {
            OffsetDateTime::now_utc()
        }
    }
}
//#[cfg(test)]
mod mockable {
    use std::sync::{Arc, RwLock};

    use serde::{de, ser};

    use super::*;

    #[derive(Debug, Clone)]
    pub struct MockableDateTime {
        time_override: Arc<RwLock<Option<OffsetDateTime>>>,
    }

    impl Serialize for MockableDateTime {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self.time_override.read() {
                Ok(lock) => match *lock {
                    Some(time) => serializer.serialize_i64(time.unix_timestamp()),
                    _ => serializer.serialize_none(),
                },
                Err(_) => Err(ser::Error::custom("lock error")),
            }
        }
    }

    impl<'de> Deserialize<'de> for MockableDateTime {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let timestamp = Option::<i64>::deserialize(deserializer)?;
            Ok(Self {
                time_override: Arc::new(RwLock::new(match timestamp {
                    Some(timestamp) => Some(
                        OffsetDateTime::from_unix_timestamp(timestamp)
                            .map_err(de::Error::custom)?,
                    ),
                    None => None,
                })),
            })
        }
    }

    impl MockableDateTime {
        pub fn now_utc(&self) -> OffsetDateTime {
            self.time_override
                .read()
                .unwrap()
                .unwrap_or_else(OffsetDateTime::now_utc)
        }
        pub fn set_override(&mut self, time_override: Option<OffsetDateTime>) {
            let mut lock = self.time_override.write().unwrap();
            *lock = time_override;
        }
    }
}
