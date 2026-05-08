import { describe, expect, test } from 'bun:test'
import { buildCompactDisplayChildren, buildTreeData, getTreeDataDisplayName } from './unpackTree'

function dir(name, children = []) {
  return {
    isDir: true,
    name,
    compressedSize: 0,
    uncompressedSize: 0,
    isCompressed: false,
    children
  }
}

function file(name) {
  return {
    isDir: false,
    name,
    hash: [1, name.length],
    compressedSize: 1,
    uncompressedSize: 1,
    isCompressed: false,
    belongsTo: 'pak',
    children: []
  }
}

function createTree(nodes) {
  return buildTreeData(nodes)
}

function collectNodeMap(nodes) {
  const map = new Map()
  const walk = (node) => {
    map.set(node.id, node)
    node.children.forEach(walk)
  }

  nodes.forEach(walk)
  return map
}

function collectBreadcrumbLabels(node, nodeMap) {
  const labels = []
  let cursor = node

  while (cursor) {
    labels.unshift(cursor.name)
    cursor = cursor.parentId ? nodeMap.get(cursor.parentId) : undefined
  }

  return labels
}

describe('compact directory display', () => {
  test('keeps the raw tree as real directory nodes', () => {
    const [root] = createTree([dir('A', [dir('B', [dir('C', [file('x.tex')])])])])

    expect(root.name).toBe('A')
    expect(root.children[0].name).toBe('B')
    expect(root.children[0].children[0].name).toBe('C')
  })

  test('compacts a single directory chain when rendering a directory children', () => {
    const [root] = createTree([dir('A', [dir('B', [dir('C', [file('x.tex')])])])])
    const [compactChild] = buildCompactDisplayChildren(root.children)

    expect(getTreeDataDisplayName(compactChild)).toBe('B / C')
    expect(compactChild.id).toBe('A/B/C')
    expect(compactChild.path).toBe('A/B/C')
  })

  test('compacts each child directory independently', () => {
    const [root] = createTree([
      dir('A', [dir('B', [dir('C', [file('x.tex')])]), dir('D', [dir('E', [file('y.tex')])])])
    ])
    const names = buildCompactDisplayChildren(root.children).map(getTreeDataDisplayName)

    expect(names).toEqual(['B / C', 'D / E'])
  })

  test('stops when a directory has files or multiple children', () => {
    const [root] = createTree([dir('A', [dir('B', [dir('C'), file('x.tex')])])])
    const [compactChild] = buildCompactDisplayChildren(root.children)

    expect(getTreeDataDisplayName(compactChild)).toBe('B')
    expect(compactChild.id).toBe('A/B')
  })

  test('compact display keeps breadcrumb data on the raw tree', () => {
    const rawTree = createTree([dir('A', [dir('B', [dir('C', [file('x.tex')])])])])
    const nodeMap = collectNodeMap(rawTree)
    const [compactChild] = buildCompactDisplayChildren(rawTree[0].children)

    expect(collectBreadcrumbLabels(nodeMap.get(compactChild.id), nodeMap)).toEqual(['A', 'B', 'C'])
  })
})
