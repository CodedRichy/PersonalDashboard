/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import "./tokens.css";
import "./app.css";

render(() => <App />, document.getElementById("root") as HTMLElement);
