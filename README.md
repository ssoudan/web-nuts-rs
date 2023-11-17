# Going NUTS with WebAssembly

What about running a NUTS sampler like [pymc-devs/nut-rs](https://github.com/pymc-devs/nuts-rs) in the browser? 

See https://ssoudan.github.io/web-nuts-rs/ for a live demo of a Bayesian regression of daily maximum temperature from NOAA Global Historical Climatology Network - Daily (GHCN-D) data all in the browser.

# Development

Use the devcontainer to get a ready to use environment.

```bash
npm i && npm run serve
```

Open http://localhost:8080/ to see the app.


