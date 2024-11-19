const formatter = new Intl.DateTimeFormat("en_gb", {
  //   timeZone: "Atlantic/Reykjavik",
  //   dateStyle: "full",
  //   timeStyle: "full",
  //   year: "2-digit",
  weekday: "short",
  day: "numeric",
  //   month: "narrow",
  hour: "2-digit",
  minute: "2-digit",
  year: "numeric",
  month: "long",
  timeZoneName: "longGeneric",
});

console.log(formatter.format(new Date()));

// console.log(formatter.calendar());
console.log(new Date());
