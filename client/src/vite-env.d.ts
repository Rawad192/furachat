/// <reference types="vite/client" />

// Déclarations de types pour les modules CSS
declare module '*.module.css' {
  const classes: { readonly [key: string]: string };
  export default classes;
}
