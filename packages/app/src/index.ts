import { mount } from "svelte";
import App from "./App.svelte";
import "./utils/theme";

document.documentElement.lang = navigator.language;

mount(App, {
    target: document.body,
});
