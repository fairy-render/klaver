use icu::{
    calendar::provider::{
        ChineseCacheV1Marker, DangiCacheV1Marker, IslamicObservationalCacheV1Marker,
        IslamicUmmAlQuraCacheV1Marker, JapaneseErasV1Marker, JapaneseExtendedErasV1Marker,
        WeekDataV1Marker,
    },
    datetime::provider::{self, calendar::*},
    decimal::provider::DecimalSymbolsV1Marker,
    plurals::provider::OrdinalV1Marker,
    timezone::provider::{MetazonePeriodV1Marker, names::IanaToBcp47MapV2Marker},
};
use icu_provider::{DataError, DataProvider, DataRequest, DataResponse};
use rquickjs::JsLifetime;
use std::sync::Arc;

pub trait ProviderTrait:
    DataProvider<TimeSymbolsV1Marker>
    + DataProvider<TimeLengthsV1Marker>
    + DataProvider<icu::datetime::provider::calendar::DateSkeletonPatternsV1Marker>
    + DataProvider<WeekDataV1Marker>
    + DataProvider<provider::time_zones::TimeZoneFormatsV1Marker>
    + DataProvider<provider::time_zones::ExemplarCitiesV1Marker>
    + DataProvider<provider::time_zones::MetazoneGenericNamesLongV1Marker>
    + DataProvider<provider::time_zones::MetazoneGenericNamesShortV1Marker>
    + DataProvider<provider::time_zones::MetazoneSpecificNamesLongV1Marker>
    + DataProvider<provider::time_zones::MetazoneSpecificNamesShortV1Marker>
    + DataProvider<OrdinalV1Marker>
    + DataProvider<DecimalSymbolsV1Marker>
    + DataProvider<BuddhistDateLengthsV1Marker>
    + DataProvider<BuddhistDateSymbolsV1Marker>
    + DataProvider<ChineseCacheV1Marker>
    + DataProvider<ChineseDateLengthsV1Marker>
    + DataProvider<ChineseDateSymbolsV1Marker>
    + DataProvider<CopticDateLengthsV1Marker>
    + DataProvider<CopticDateSymbolsV1Marker>
    + DataProvider<DangiCacheV1Marker>
    + DataProvider<DangiDateLengthsV1Marker>
    + DataProvider<DangiDateSymbolsV1Marker>
    + DataProvider<EthiopianDateLengthsV1Marker>
    + DataProvider<EthiopianDateSymbolsV1Marker>
    + DataProvider<GregorianDateLengthsV1Marker>
    + DataProvider<GregorianDateSymbolsV1Marker>
    + DataProvider<HebrewDateLengthsV1Marker>
    + DataProvider<HebrewDateSymbolsV1Marker>
    + DataProvider<IndianDateLengthsV1Marker>
    + DataProvider<IndianDateSymbolsV1Marker>
    + DataProvider<IslamicDateLengthsV1Marker>
    + DataProvider<IslamicDateSymbolsV1Marker>
    + DataProvider<IslamicObservationalCacheV1Marker>
    + DataProvider<IslamicUmmAlQuraCacheV1Marker>
    + DataProvider<JapaneseDateLengthsV1Marker>
    + DataProvider<JapaneseDateSymbolsV1Marker>
    + DataProvider<JapaneseErasV1Marker>
    + DataProvider<JapaneseExtendedDateLengthsV1Marker>
    + DataProvider<JapaneseExtendedDateSymbolsV1Marker>
    + DataProvider<JapaneseExtendedErasV1Marker>
    + DataProvider<PersianDateLengthsV1Marker>
    + DataProvider<PersianDateSymbolsV1Marker>
    + DataProvider<RocDateLengthsV1Marker>
    + DataProvider<RocDateSymbolsV1Marker>
    + DataProvider<IanaToBcp47MapV2Marker>
    + DataProvider<MetazonePeriodV1Marker>
{
}

impl<T> ProviderTrait for T where
    T: DataProvider<TimeSymbolsV1Marker>
        + DataProvider<TimeLengthsV1Marker>
        + DataProvider<icu::datetime::provider::calendar::DateSkeletonPatternsV1Marker>
        + DataProvider<WeekDataV1Marker>
        + DataProvider<provider::time_zones::TimeZoneFormatsV1Marker>
        + DataProvider<provider::time_zones::ExemplarCitiesV1Marker>
        + DataProvider<provider::time_zones::MetazoneGenericNamesLongV1Marker>
        + DataProvider<provider::time_zones::MetazoneGenericNamesShortV1Marker>
        + DataProvider<provider::time_zones::MetazoneSpecificNamesLongV1Marker>
        + DataProvider<provider::time_zones::MetazoneSpecificNamesShortV1Marker>
        + DataProvider<OrdinalV1Marker>
        + DataProvider<DecimalSymbolsV1Marker>
        + DataProvider<BuddhistDateLengthsV1Marker>
        + DataProvider<BuddhistDateSymbolsV1Marker>
        + DataProvider<ChineseCacheV1Marker>
        + DataProvider<ChineseDateLengthsV1Marker>
        + DataProvider<ChineseDateSymbolsV1Marker>
        + DataProvider<CopticDateLengthsV1Marker>
        + DataProvider<CopticDateSymbolsV1Marker>
        + DataProvider<DangiCacheV1Marker>
        + DataProvider<DangiDateLengthsV1Marker>
        + DataProvider<DangiDateSymbolsV1Marker>
        + DataProvider<EthiopianDateLengthsV1Marker>
        + DataProvider<EthiopianDateSymbolsV1Marker>
        + DataProvider<GregorianDateLengthsV1Marker>
        + DataProvider<GregorianDateSymbolsV1Marker>
        + DataProvider<HebrewDateLengthsV1Marker>
        + DataProvider<HebrewDateSymbolsV1Marker>
        + DataProvider<IndianDateLengthsV1Marker>
        + DataProvider<IndianDateSymbolsV1Marker>
        + DataProvider<IslamicDateLengthsV1Marker>
        + DataProvider<IslamicDateSymbolsV1Marker>
        + DataProvider<IslamicObservationalCacheV1Marker>
        + DataProvider<IslamicUmmAlQuraCacheV1Marker>
        + DataProvider<JapaneseDateLengthsV1Marker>
        + DataProvider<JapaneseDateSymbolsV1Marker>
        + DataProvider<JapaneseErasV1Marker>
        + DataProvider<JapaneseExtendedDateLengthsV1Marker>
        + DataProvider<JapaneseExtendedDateSymbolsV1Marker>
        + DataProvider<JapaneseExtendedErasV1Marker>
        + DataProvider<PersianDateLengthsV1Marker>
        + DataProvider<PersianDateSymbolsV1Marker>
        + DataProvider<RocDateLengthsV1Marker>
        + DataProvider<RocDateSymbolsV1Marker>
        + DataProvider<IanaToBcp47MapV2Marker>
        + DataProvider<MetazonePeriodV1Marker>
{
}

#[derive(Clone)]
pub struct DynProvider {
    provider: Arc<dyn ProviderTrait>,
}

unsafe impl<'js> JsLifetime<'js> for DynProvider {
    type Changed<'to> = DynProvider;
}

impl DynProvider {
    pub fn new<P: 'static>(provider: P) -> DynProvider
    where
        P: DataProvider<TimeSymbolsV1Marker>
            + DataProvider<TimeLengthsV1Marker>
            + DataProvider<icu::datetime::provider::calendar::DateSkeletonPatternsV1Marker>
            + DataProvider<WeekDataV1Marker>
            + DataProvider<provider::time_zones::TimeZoneFormatsV1Marker>
            + DataProvider<provider::time_zones::ExemplarCitiesV1Marker>
            + DataProvider<provider::time_zones::MetazoneGenericNamesLongV1Marker>
            + DataProvider<provider::time_zones::MetazoneGenericNamesShortV1Marker>
            + DataProvider<provider::time_zones::MetazoneSpecificNamesLongV1Marker>
            + DataProvider<provider::time_zones::MetazoneSpecificNamesShortV1Marker>
            + DataProvider<OrdinalV1Marker>
            + DataProvider<DecimalSymbolsV1Marker>
            + DataProvider<BuddhistDateLengthsV1Marker>
            + DataProvider<BuddhistDateSymbolsV1Marker>
            + DataProvider<ChineseCacheV1Marker>
            + DataProvider<ChineseDateLengthsV1Marker>
            + DataProvider<ChineseDateSymbolsV1Marker>
            + DataProvider<CopticDateLengthsV1Marker>
            + DataProvider<CopticDateSymbolsV1Marker>
            + DataProvider<DangiCacheV1Marker>
            + DataProvider<DangiDateLengthsV1Marker>
            + DataProvider<DangiDateSymbolsV1Marker>
            + DataProvider<EthiopianDateLengthsV1Marker>
            + DataProvider<EthiopianDateSymbolsV1Marker>
            + DataProvider<GregorianDateLengthsV1Marker>
            + DataProvider<GregorianDateSymbolsV1Marker>
            + DataProvider<HebrewDateLengthsV1Marker>
            + DataProvider<HebrewDateSymbolsV1Marker>
            + DataProvider<IndianDateLengthsV1Marker>
            + DataProvider<IndianDateSymbolsV1Marker>
            + DataProvider<IslamicDateLengthsV1Marker>
            + DataProvider<IslamicDateSymbolsV1Marker>
            + DataProvider<IslamicObservationalCacheV1Marker>
            + DataProvider<IslamicUmmAlQuraCacheV1Marker>
            + DataProvider<JapaneseDateLengthsV1Marker>
            + DataProvider<JapaneseDateSymbolsV1Marker>
            + DataProvider<JapaneseErasV1Marker>
            + DataProvider<JapaneseExtendedDateLengthsV1Marker>
            + DataProvider<JapaneseExtendedDateSymbolsV1Marker>
            + DataProvider<JapaneseExtendedErasV1Marker>
            + DataProvider<PersianDateLengthsV1Marker>
            + DataProvider<PersianDateSymbolsV1Marker>
            + DataProvider<RocDateLengthsV1Marker>
            + DataProvider<RocDateSymbolsV1Marker>
            + DataProvider<IanaToBcp47MapV2Marker>
            + DataProvider<MetazonePeriodV1Marker>,
    {
        DynProvider {
            provider: Arc::new(provider),
        }
    }
}

impl DataProvider<TimeSymbolsV1Marker> for DynProvider {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<TimeSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<TimeSymbolsV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<TimeLengthsV1Marker> for DynProvider {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<TimeLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<TimeLengthsV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<icu::datetime::provider::calendar::DateSkeletonPatternsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<
        DataResponse<icu::datetime::provider::calendar::DateSkeletonPatternsV1Marker>,
        DataError,
    > {
        <Arc<dyn ProviderTrait> as DataProvider<
            icu::datetime::provider::calendar::DateSkeletonPatternsV1Marker,
        >>::load(&self.provider, req)
    }
}

impl DataProvider<WeekDataV1Marker> for DynProvider {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<WeekDataV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<WeekDataV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<provider::time_zones::TimeZoneFormatsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::TimeZoneFormatsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<provider::time_zones::TimeZoneFormatsV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<provider::time_zones::ExemplarCitiesV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::ExemplarCitiesV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<provider::time_zones::ExemplarCitiesV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<provider::time_zones::MetazoneGenericNamesLongV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::MetazoneGenericNamesLongV1Marker>, DataError>
    {
        <Arc<dyn ProviderTrait> as DataProvider<
            provider::time_zones::MetazoneGenericNamesLongV1Marker,
        >>::load(&self.provider, req)
    }
}

impl DataProvider<provider::time_zones::MetazoneGenericNamesShortV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::MetazoneGenericNamesShortV1Marker>, DataError>
    {
        <Arc<dyn ProviderTrait> as DataProvider<
            provider::time_zones::MetazoneGenericNamesShortV1Marker,
        >>::load(&self.provider, req)
    }
}

impl DataProvider<provider::time_zones::MetazoneSpecificNamesLongV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::MetazoneSpecificNamesLongV1Marker>, DataError>
    {
        <Arc<dyn ProviderTrait> as DataProvider<
            provider::time_zones::MetazoneSpecificNamesLongV1Marker,
        >>::load(&self.provider, req)
    }
}

impl DataProvider<provider::time_zones::MetazoneSpecificNamesShortV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<provider::time_zones::MetazoneSpecificNamesShortV1Marker>, DataError>
    {
        <Arc<dyn ProviderTrait> as DataProvider<
            provider::time_zones::MetazoneSpecificNamesShortV1Marker,
        >>::load(&self.provider, req)
    }
}

impl DataProvider<OrdinalV1Marker> for DynProvider {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<OrdinalV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<OrdinalV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<DecimalSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<DecimalSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<DecimalSymbolsV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<BuddhistDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<BuddhistDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<BuddhistDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<BuddhistDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<BuddhistDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<BuddhistDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<ChineseCacheV1Marker> for DynProvider {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<ChineseCacheV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<ChineseCacheV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<ChineseDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<ChineseDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<ChineseDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<ChineseDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<ChineseDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<ChineseDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<CopticDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<CopticDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<CopticDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<CopticDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<CopticDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<CopticDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<DangiCacheV1Marker> for DynProvider {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<DangiCacheV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<DangiCacheV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<DangiDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<DangiDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<DangiDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<DangiDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<DangiDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<DangiDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<EthiopianDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<EthiopianDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<EthiopianDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<EthiopianDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<EthiopianDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<EthiopianDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<GregorianDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<GregorianDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<GregorianDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<GregorianDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<GregorianDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<GregorianDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<HebrewDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<HebrewDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<HebrewDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<HebrewDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<HebrewDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<HebrewDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<IndianDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IndianDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<IndianDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<IndianDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IndianDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<IndianDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<IslamicDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IslamicDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<IslamicDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<IslamicDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IslamicDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<IslamicDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<IslamicObservationalCacheV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IslamicObservationalCacheV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<IslamicObservationalCacheV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<IslamicUmmAlQuraCacheV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<IslamicUmmAlQuraCacheV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<IslamicUmmAlQuraCacheV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<JapaneseDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<JapaneseDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<JapaneseDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<JapaneseDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<JapaneseErasV1Marker> for DynProvider {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<JapaneseErasV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<JapaneseErasV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<JapaneseExtendedDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseExtendedDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<JapaneseExtendedDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<JapaneseExtendedDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseExtendedDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<JapaneseExtendedDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<JapaneseExtendedErasV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<JapaneseExtendedErasV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<JapaneseExtendedErasV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<PersianDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<PersianDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<PersianDateLengthsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<PersianDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<PersianDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<PersianDateSymbolsV1Marker>>::load(
            &self.provider,
            req,
        )
    }
}

impl DataProvider<RocDateLengthsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<RocDateLengthsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<RocDateLengthsV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<RocDateSymbolsV1Marker> for DynProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<RocDateSymbolsV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<RocDateSymbolsV1Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<IanaToBcp47MapV2Marker> for DynProvider {
    fn load(&self, req: DataRequest) -> Result<DataResponse<IanaToBcp47MapV2Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<IanaToBcp47MapV2Marker>>::load(&self.provider, req)
    }
}

impl DataProvider<MetazonePeriodV1Marker> for DynProvider {
    fn load(&self, req: DataRequest) -> Result<DataResponse<MetazonePeriodV1Marker>, DataError> {
        <Arc<dyn ProviderTrait> as DataProvider<MetazonePeriodV1Marker>>::load(&self.provider, req)
    }
}
