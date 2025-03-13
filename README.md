# uiua-node

This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon).

## Building uiua-node

```sh
$ yarn build
```

## Exploring uiua-node

After building uiua-node, you can explore its exports at the Node console:

```
$ yarn
$ yarn build
$ node
> require('.').eval_mm("$ Hello, world!")
{ outputs: [ { type: 'stdout', content: '"Hello, world!"\n' } ] }
```
