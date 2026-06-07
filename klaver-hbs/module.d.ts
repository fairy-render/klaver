export class Handlebars {
  render(name: string, data: Record<string, unknown>): string;
  registerTemplate(name: string, template: string): void;
}
