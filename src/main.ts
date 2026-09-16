import "./ui/style.css";
import { GameApp } from "./app";
const app = new GameApp(document.querySelector<HTMLElement>("#app")!);
void app.boot();
