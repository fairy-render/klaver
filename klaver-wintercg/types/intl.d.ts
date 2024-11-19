export namespace Intl {
	export class Locale {}

	export interface DateTimeFormatOptions {
		timeZone?: string;
		calendar?: string;
		//
		dateStyle?: "full" | "long" | "medium" | "short";
		timeStyle?: "full" | "long" | "medium" | "short";
		hour?: "numeric" | "2-digit";
		minute?: "numeric" | "2-digit";
		month?: "long" | "short" | "narrow" | "numeric" | "2-digit";
		year?: "numeric" | "2-digit";
		second?: "numeric" | "2digit";
		timeZoneName?: "short" | "long" | "shortGeneric" | "longGeneric";
		weekday?: "long" | "short" | "narrow";
		era?: "long" | "short" | "narrow";
		dayPeriod: "long" | "short" | "narrow";
	}

	export class DateTimeFormat {
		constructor(
			locales?: string | Locale | (string | Locale)[],
			options?: DateTimeFormatOptions,
		);

		format(date: Date): string;
	}
}
