window.propolisDarkMode = false;

if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    window.propolisDarkMode = true;
    console.log("Dark mode is: " + window.propolisDarkMode);
}
