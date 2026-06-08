use icu::{
    calendar::provider::{
            ChineseCacheV1Marker, DangiCacheV1Marker, IslamicObservationalCacheV1Marker,
            IslamicUmmAlQuraCacheV1Marker, JapaneseErasV1Marker, JapaneseExtendedErasV1Marker,
            WeekDataV1Marker,
        },
    datetime::provider::{self, calendar::*},
    decimal::provider::DecimalSymbolsV1Marker,
    plurals::provider::OrdinalV1Marker,
    timezone::provider::{
        names::IanaToBcp47MapV2Marker,
        MetazonePeriodV1Marker,
    },
};
use icu_provider::{DataError, DataProvider, DataRequest, DataResponse};

pub struct Baked {
    datetime: icu::datetime::provider::Baked,
    calendar: icu::calendar::provider::Baked,
    plurals: icu::plurals::provider::Baked,
    decimal: icu::decimal::provider::Baked,
    timezone: icu::timezone::provider::Baked,
}

impl Baked {
    pub const fn new() -> Baked {
        Baked {
            datetime: icu::datetime::provider::Baked,
            calendar: icu::calendar::provider::Baked,
            plurals: icu::plurals::provider::Baked,
            decimal: icu::decimal::provider::Baked,
            timezone: icu::timezone::provider::Baked,
        }
    }
}

impl DataProvider<TimeSymbolsV1Marker> for Baked {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<TimeSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<TimeLengthsV1Marker> for Baked {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<TimeLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<icu::datetime::provider::calendar::DateSkeletonPatternsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<
        DataResponse<icu::datetime::provider::calendar::DateSkeletonPatternsV1Marker>,
        DataError,
    > {
        self.datetime.load(req)
    }
}

impl DataProvider<WeekDataV1Marker> for Baked {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<WeekDataV1Marker>, DataError> {
        self.calendar.load(req)
    }
}

impl DataProvider<provider::time_zones::TimeZoneFormatsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::TimeZoneFormatsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<provider::time_zones::ExemplarCitiesV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::ExemplarCitiesV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<provider::time_zones::MetazoneGenericNamesLongV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::MetazoneGenericNamesLongV1Marker>, DataError>
    {
        self.datetime.load(req)
    }
}

impl DataProvider<provider::time_zones::MetazoneGenericNamesShortV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::MetazoneGenericNamesShortV1Marker>, DataError>
    {
        self.datetime.load(req)
    }
}

impl DataProvider<provider::time_zones::MetazoneSpecificNamesLongV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::MetazoneSpecificNamesLongV1Marker>, DataError>
    {
        self.datetime.load(req)
    }
}

impl DataProvider<provider::time_zones::MetazoneSpecificNamesShortV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::MetazoneSpecificNamesShortV1Marker>, DataError>
    {
        self.datetime.load(req)
    }
}

impl DataProvider<OrdinalV1Marker> for Baked {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<OrdinalV1Marker>, DataError> {
        self.plurals.load(req)
    }
}

impl DataProvider<DecimalSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<DecimalSymbolsV1Marker>, DataError> {
        self.decimal.load(req)
    }
}

impl DataProvider<BuddhistDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<BuddhistDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<BuddhistDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<BuddhistDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<ChineseCacheV1Marker> for Baked {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<ChineseCacheV1Marker>, DataError> {
        self.calendar.load(req)
    }
}

impl DataProvider<ChineseDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<ChineseDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<ChineseDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<ChineseDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<CopticDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<CopticDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<CopticDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<CopticDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<DangiCacheV1Marker> for Baked {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<DangiCacheV1Marker>, DataError> {
        self.calendar.load(req)
    }
}

impl DataProvider<DangiDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<DangiDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<DangiDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<DangiDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<EthiopianDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<EthiopianDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<EthiopianDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<EthiopianDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<GregorianDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<GregorianDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<GregorianDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<GregorianDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<HebrewDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<HebrewDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<HebrewDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<HebrewDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<IndianDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IndianDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<IndianDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IndianDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<IslamicDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IslamicDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<IslamicDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IslamicDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<IslamicObservationalCacheV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IslamicObservationalCacheV1Marker>, DataError> {
        self.calendar.load(req)
    }
}

impl DataProvider<IslamicUmmAlQuraCacheV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IslamicUmmAlQuraCacheV1Marker>, DataError> {
        self.calendar.load(req)
    }
}

impl DataProvider<JapaneseDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<JapaneseDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<JapaneseErasV1Marker> for Baked {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<JapaneseErasV1Marker>, DataError> {
        self.calendar.load(req)
    }
}

impl DataProvider<JapaneseExtendedDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseExtendedDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<JapaneseExtendedDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseExtendedDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<JapaneseExtendedErasV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseExtendedErasV1Marker>, DataError> {
        self.calendar.load(req)
    }
}

impl DataProvider<PersianDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<PersianDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<PersianDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<PersianDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<RocDateLengthsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<RocDateLengthsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<RocDateSymbolsV1Marker> for Baked {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<RocDateSymbolsV1Marker>, DataError> {
        self.datetime.load(req)
    }
}

impl DataProvider<IanaToBcp47MapV2Marker> for Baked {
    fn load(&self, req: DataRequest) -> Result<DataResponse<IanaToBcp47MapV2Marker>, DataError> {
        self.timezone.load(req)
    }
}

impl DataProvider<MetazonePeriodV1Marker> for Baked {
    fn load(&self, req: DataRequest) -> Result<DataResponse<MetazonePeriodV1Marker>, DataError> {
        self.timezone.load(req)
    }
}
