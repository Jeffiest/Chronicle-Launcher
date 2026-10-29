import "@fontsource/cinzel/500.css";
import "@fontsource/cinzel/600.css";
import "@fontsource-variable/inter";
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

mount(App, { target: document.getElementById("app")! });
