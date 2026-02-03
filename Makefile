build-website:
	cd apps/web && wasm-pack build --target web
	mkdir -p build
	cp apps/web/pkg/swipe_web.js apps/web/pkg/swipe_web_bg.wasm build/
	cp apps/web/www/index.html build/

serve: build-website
	cd build && python3 -m http.server 8000
