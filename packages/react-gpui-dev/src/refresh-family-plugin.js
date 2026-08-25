module.exports = function refreshFamilyPlugin({ types: t }) {
  return {
    name: "react-gpui-stable-family",
    visitor: {
      FunctionDeclaration(path, state) {
        const name = path.node.id && path.node.id.name;
        if (!name || !/^[A-Z]/.test(name)) return;
        const implementationName = path.scope.generateUidIdentifier(`${name}Implementation`);
        const signatureName = path.scope.generateUidIdentifier(`${name}Signature`);
        const hookNames = [];
        path.traverse({
          CallExpression(callPath) {
            if (callPath.findParent((parent) => parent.isFunction() && parent.node !== path.node)) {
              return;
            }
            const callee = callPath.node.callee;
            const hookName =
              callee.type === "Identifier"
                ? callee.name
                : callee.type === "MemberExpression" && callee.property.type === "Identifier"
                  ? callee.property.name
                  : undefined;
            if (hookName && /^use[A-Z]/.test(hookName)) hookNames.push(hookName);
          },
        });
        const implementation = t.functionExpression(
          implementationName,
          path.node.params,
          path.node.body,
          path.node.generator,
          path.node.async,
        );
        implementation.body.body.unshift(
          t.expressionStatement(t.callExpression(signatureName, [])),
        );
        const moduleId = `${state.filename}:${name}`;
        const family = t.callExpression(t.identifier("__reactGpuiFamily"), [
          t.stringLiteral(moduleId),
          implementation,
          t.stringLiteral(hookNames.join("|")),
        ]);
        const signatureDeclaration = t.variableDeclaration("const", [
          t.variableDeclarator(
            signatureName,
            t.callExpression(t.identifier("$RefreshSig$"), []),
          ),
        ]);
        const declaration = t.variableDeclaration("const", [
          t.variableDeclarator(t.identifier(name), family),
        ]);
        const parent = path.parentPath;
        if (parent.isExportDefaultDeclaration()) {
          parent.replaceWithMultiple([
            signatureDeclaration,
            declaration,
            t.exportDefaultDeclaration(t.identifier(name)),
          ]);
        } else if (parent.isExportNamedDeclaration()) {
          parent.replaceWithMultiple([
            signatureDeclaration,
            t.exportNamedDeclaration(declaration),
          ]);
        } else {
          path.replaceWithMultiple([signatureDeclaration, declaration]);
        }
      },
    },
  };
};
