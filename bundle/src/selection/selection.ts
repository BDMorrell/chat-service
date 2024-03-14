import css from './selection.scss?url';

const nodeView = document.createElement("div");
nodeView.id = "nodeView";
const breadCrumbs = {
  container: document.createElement("div"),
  anchor: document.createElement("ol"),
  focus: document.createElement("ol"),
}
breadCrumbs.container.id = "breadCrumbs";
const mainView = createDivWithBody();
mainView.id = "mainView";
mainView.contentEditable = "true";

(() => {
  document.body.append(mainView, nodeView, breadCrumbs.container);
  const cssLink = document.createElement("link");
  cssLink.rel = "stylesheet";
  cssLink.href = css;
  document.head.appendChild(cssLink);
})();

readyBreadcrumbView();

document.addEventListener("selectionchange", handleSelection);
handleSelection();

function readyBreadcrumbView() {
  const divAnchor = document.createElement("div");
  const divFocus = document.createElement("div");
  divAnchor.append(new Text("Anchor: "), breadCrumbs.anchor);
  divFocus.append(new Text("Focus: "), breadCrumbs.focus);
  breadCrumbs.anchor.classList.add("breadcrumb");
  breadCrumbs.focus.classList.add("breadcrumb");
  divAnchor.appendChild(breadCrumbs.anchor);
  divFocus.appendChild(breadCrumbs.focus);
  breadCrumbs.container.append(divAnchor, divFocus);
}

function createDivWithBody() {
  const container = document.createElement("div");
  container.append(...extractAllChildren(document.body));
  return container;
}

function handleSelection() {
  const selection = document.getSelection();
  extractAllChildren(breadCrumbs.anchor);
  extractAllChildren(breadCrumbs.focus);
  if (selection) {
    breadCrumbs.anchor.append(...ancestryCrumbs(selection.anchorNode, selection.anchorOffset));
    breadCrumbs.focus.append(...ancestryCrumbs(selection.focusNode, selection.focusOffset));
  } else {
    for (const list of [breadCrumbs.anchor, breadCrumbs.focus]) {
      const item = document.createElement("li");
      item.appendChild(new Text("<No selection>"));
      list.appendChild(item);
    }
  }
}

function extractAllChildren(node: Node): Array<Node> {
  const removed = new Array();
  while (node.hasChildNodes()) {
    removed.push(node.removeChild(node.firstChild!));
  }
  return removed;
}

function ancestryCrumbs(node: Node | null, selectionOffset: Number): Array<HTMLLIElement> {
  const TEXT_NODE = 3; // from JavaScript documentation
  var trail: Array<HTMLLIElement> = new Array();
  if (node) {
    var cursor: Node | ParentNode | null = node;
    var child: Node | null = null;
    while (cursor) {
      const crumb = document.createElement("li");
      let message = cursor.nodeName.toLocaleLowerCase();
      if (child && cursor.childNodes.length > 1) {
        const len = cursor.childNodes.length;
        for (const [idx, nod] of cursor.childNodes.entries()) {
          if (nod.isSameNode(child)) {
            message += `[${(idx + 1).toString()}/${len}]`;
            break;
          }
        }
      } else if (child === null && cursor.nodeType === TEXT_NODE) {
        message += `[${selectionOffset}/${(cursor as Text).nodeValue?.length}]`;
      }
      crumb.appendChild(new Text(message));
      trail.push(crumb);
      child = cursor;
      cursor = cursor.parentNode;
    }
  } else {
    const crumb = document.createElement("li");
    crumb.appendChild(new Text("<NULL>"));
    trail.push(crumb);
  }
  return trail.reverse();
}

