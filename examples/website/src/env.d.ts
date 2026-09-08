declare module "babel-plugin-jsx-dom-expressions" {
  const plugin: import("@babel/standalone").PluginItem;
  export default plugin;
}

declare module "*?raw" {
  const source: string;
  export default source;
}
