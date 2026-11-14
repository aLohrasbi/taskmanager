import { mount } from 'svelte';
import App from './App.svelte';
import './styles/tokens.css';
import './styles/global.css';

export default mount(App, { target: document.getElementById('app')! });