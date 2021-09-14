# Design Notes

## Dedicated error page

Like text page

- with dynamic content
- Limited limited page transition e.g. to SystemStart/Home only
- forbidden User Interaction

## Return of dispatch function

- Thesis: HMI Exit is rather an ordnary use case than an error?

## Startup and Shutdown as additional data type variants of page manager

- Intro of Builder pattern for page manager
- Factor startup page and shutdown page from page manager into dedicated
  data;
- more lean code?
- feature toggles at build time.
