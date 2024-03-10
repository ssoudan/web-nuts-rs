# Going NUTS with WebAssembly

What about running a NUTS sampler like [pymc-devs/nuts-rs](https://github.com/pymc-devs/nuts-rs) in the browser? 

See https://ssoudan.github.io/web-nuts-rs/ for a live demo of a Bayesian regression of daily maximum temperature from NOAA Global Historical Climatology Network - Daily (GHCN-D) data all in the browser.

# Development

Use the devcontainer to get a ready to use environment.

To run the app in development mode, run:
```bash
npm i && npm run serve
```

To build the app for production, run:
```bash
npm i && npm run serve:prod
```

Either way, you can now navigate to `http://localhost:8080` to view the app.
