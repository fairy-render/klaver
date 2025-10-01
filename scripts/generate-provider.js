const v = `DataProvider<TimeSymbolsV1Marker>
        + DataProvider<TimeLengthsV1Marker>
        + DataProvider<icu_datetime::provider::calendar::DateSkeletonPatternsV1Marker>
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
        + DataProvider<RocDateSymbolsV1Marker>`
  .split("+")
  .map((m) =>
    m.trim().replace("DataProvider", "").replace("<", "").replace(">", "")
  );

for (const n of v) {
  const template = `
impl DataProvider<${n}> for DynProvider {
    fn load(&self, req: DataRequest<'_>) -> Result<DataResponse<${n}>, DataError> {
        <Box<dyn ProviderTrait> as DataProvider<${n}>>::load(&self.provider, req)
    }
}

`;

  console.log(template);
}
