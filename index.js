import "./styles.css";

// show a loading message while we load the wasm
const status = document.getElementById("status");
status.textContent = "Loading WASM...";

// load the wasm
import('./pkg')
    .then(wasm => {        
        const main = document.getElementById("main");        
   
        const start_button = document.getElementById("start_button");

        const seed_input = document.getElementById("seed");
        const chain_count_input = document.getElementById("chain_count");
        const chain_count_display = document.getElementById("chain_count_display");
        const results = document.getElementById("results");
        const tuning = document.getElementById("tuning");
        const samples = document.getElementById("samples");
        const raw = document.getElementById("raw_text");
        const input = document.getElementById("input_text");
        const input_url = document.getElementById("input_url");
        const load_button = document.getElementById("load_button");
        const posterior = document.getElementById("posterior");

        const int = (str) => {
            const i = parseInt(str);
            if (isNaN(i)) {
                return 0;
            }
            return i;
        }

        const clear_data = () => {
            try{
                // clear the canvas
                const canvas = document.getElementById("trace_plot");
                const ctx = canvas.getContext("2d");
                ctx.clearRect(0, 0, canvas.width, canvas.height);

                // clear the posterior
                posterior.textContent = "";

            } catch (e) {
                console.error(e);
                input.value = "Error: " + e
            }; 
        }

        const plot = () => {
            try {
                console.log("plotting");                
                wasm.plot_tmax("plot", posterior.textContent, input.value);
            } catch (e) {
                console.error(e);
                input.value = "Error: " + e
            }; 
        }

        const sample = (onSuccess) => {
            status.textContent = "Running...";

            setTimeout(() => {
                const start = Date.now();   

                const chain_count = BigInt(chain_count_display.value);
                const seed = BigInt(seed_input.value);

                const input_data = input.value;
                const tuning_value = BigInt(tuning.value);
                const samples_value = BigInt(samples.value);

                console.log(`input:\n${input_data}`);
                console.log(`seed: ${seed}`);
                console.log(`chain_count: ${chain_count}`);
                console.log(`tuning: ${tuning_value}`);
                console.log(`samples: ${samples_value}`);
                
                setTimeout(() => {
                    wasm.run_with("trace_plot", "posterior", seed, input_data, chain_count, tuning_value, samples_value);
                        const end = Date.now();
                        const elapsed = end - start;
                        results.textContent = `Elapsed: ${elapsed}ms`;     
                        
                        setTimeout(() => {
                            plot();

                            if (onSuccess) {
                                onSuccess();
                            }
                            status.textContent = "Ready";
                    }, 10);
                }, 10);
            }, 10);
        }

        const prepare = (onSuccess) => {
            status.textContent = "Preparing...";
            try {
                // clear the canvas
                const canvas = document.getElementById("trace_plot");
                const ctx = canvas.getContext("2d");
                ctx.clearRect(0, 0, canvas.width, canvas.height);

                input.value = wasm.prepare(raw.value);

                if (onSuccess) {
                    onSuccess();
                }
            } catch (e) {
                console.error(e);
                input.value = "Error: " + e
            }; 
        }

        const prepare_and_sample = (onSuccess) => {
            prepare(() => {
                setTimeout(() => {
                    clear_data();
                    
                    plot();

                    setTimeout(() => {
                        sample(onSuccess);                                                           
                    }, 10);
                }, 10);
            });
        }

        const load = (url, onSuccess) => {
            // disable the buttons
            load_button.disabled = true;
            start_button.disabled = true;

            // clear the results
            results.textContent = "";

            setTimeout(() => {
                try {
                    status.textContent = `Fetching ${url}`;

                    setTimeout(async() => {
                        try {
                            const response = await fetch(url);
                            const text = await response.text();
                            raw.value = text;

                            if (onSuccess) {
                                onSuccess(() => {
                                    status.textContent = 'Ready';

                                    // enable the button
                                    load_button.disabled = false;
                                    start_button.disabled = false;
                                });
                            }
                        } catch (e) {
                            console.error(e);
                            input.value = "Error: " + e
                        } 
                    }, 10);

                } catch (e) {
                    console.error(e);
                    input.value = "Error: " + e
                }; 
                
            }, 10);
        }

        load_button.onclick = () => {
            const url = input_url.value;

            load(url, (onSuccess) => {
                prepare_and_sample(onSuccess);                    
            });
        }

        // on_change of raw, update input
        // raw.onchange = () => {
            // console.log("raw changed");
            // prepare_and_run();
        // }

        prepare_and_sample(() => {
            const new_seed = Math.floor(Math.random() * 10000);
            seed_input.value = new_seed.toString();
        });

       // update chain_count_display on chain_count change
        chain_count_input.onchange = () => {
            const chain_count = int(chain_count_input.value);
            chain_count_display.value = chain_count.toString();
        } 

        status.textContent = "Ready";

        const run = (onSuccess) => {
            // disable the buttons
            load_button.disabled = true;
            start_button.disabled = true;            

            // clear the results
            results.textContent = "";

            prepare_and_sample(() => {
                if (onSuccess) {
                    onSuccess();
                    
                    // enable the button
                    load_button.disabled = false;
                    start_button.disabled = false;

                }
            });
        }

        start_button.onclick = () => {
   
            run(() => {// update the seed
                const new_seed = Math.floor(Math.random() * 10000);
                seed_input.value = new_seed.toString();
            });                
        }
       
    })
    .catch(console.error);
