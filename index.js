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

        const int = (str) => {
            const i = parseInt(str);
            if (isNaN(i)) {
                return 0;
            }
            return i;
        }

        const run = () => {
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

            wasm.run_with("trace_plot", seed, input_data, chain_count, tuning_value, samples_value);
            
            const end = Date.now();
            const elapsed = end - start;
            results.textContent = `Elapsed: ${elapsed}ms`;  
        }

        const prepare_and_run = () => {
            try {
                console.log("preparing");
                input.value = wasm.prepare(raw.value);

                // plot the input
                console.log("plotting");
                wasm.plot_tmax("plot", input.value);
                // run
                console.log("running");
                run();
            } catch (e) {
                console.error(e);
                input.value = "Error: " + e
            }; 
        }

        load_button.onclick = async() => {
            const url = input_url.value;

            // fetch the data
            console.log(`Fetching ${url}`);
            const response = await fetch(url);
            const text = await response.text();
            raw.value = text;

            prepare_and_run();
        }

        // on_change of raw, update input
        raw.onchange = () => {
            console.log("raw changed");
            prepare_and_run();
        }

        prepare_and_run();

       // update chain_count_display on chain_count change
        chain_count_input.onchange = () => {
            const chain_count = int(chain_count_input.value);
            chain_count_display.value = chain_count.toString();
        } 

    
        status.textContent = "Ready. Click the button to start.";

        start_button.onclick = () => {                                      
            run();

            // update the seed
            const new_seed = Math.floor(Math.random() * 10000);
            seed_input.value = new_seed.toString();
        }
       
    })
    .catch(console.error);
