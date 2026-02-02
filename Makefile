build-website:
	wasm-pack build --target web
	mkdir -p build
	cp pkg/swipe_predictor_rs.js pkg/swipe_predictor_rs_bg.wasm build/
	cp src/index.html build/

serve: build-website
	cd build && python3 -m http.server 8000
