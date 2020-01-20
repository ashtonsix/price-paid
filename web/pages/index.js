import React, {useEffect, useRef, useState} from 'react'
import createPalette from 'color-interpolate'

const _palette = [
  '#665191',
  '#a05195',
  '#d45087',
  '#f95d6a',
  '#ff7c43',
  '#ffa600',
  '#dfbd0c',
  '#becf36',
  '#9bdd5f',
  '#76e889',
  '#4cf0b4',
  '#14f7dc',
  '#00fbff',
  '#00fbff',
  '#4ffaff',
  '#74f9ff',
  '#91f8ff',
  '#aaf7ff',
  '#c0f5ff',
  '#d3f4ff',
  '#e3f4ff'
]

const palette = createPalette(_palette)

const gradient =
  'linear-gradient(90deg, ' +
  _palette
    .map((c, i) => {
      const p = (i / (_palette.length - 1)) * 100
      return `${c} ${p.toFixed(2)}%`
    })
    .join(', ') +
  ')'

const MAX = 2000 * 1000

const ppTilePrefix =
  'https://ashtonsix-price-paid.s3-eu-west-1.amazonaws.com/tiles/'

function range2D(x0, x1, y0, y1) {
  const range = []
  for (let x = x0; x <= x1; x++) {
    for (let y = y0; y <= y1; y++) {
      range.push([x, y])
    }
  }
  return range
}

function shuffle(a) {
  var j, x, i
  for (i = a.length - 1; i > 0; i--) {
    j = Math.floor(Math.random() * (i + 1))
    x = a[i]
    a[i] = a[j]
    a[j] = x
  }
  return a
}

const tileData = {}
const drawn = new Set()

async function doPricePaid(map) {
  map = await map
  const bounds = map.getBounds()
  const lat0 = Math.floor((bounds._southWest.lat + 90) * 100)
  const lat1 = Math.floor((bounds._northEast.lat + 90) * 100 + 1)
  const lon0 = Math.floor((bounds._southWest.lng + 180) * 100)
  const lon1 = Math.floor((bounds._northEast.lng + 180) * 100 + 1)
  const tileCount = (lat1 - lat0 + 1) * (lon1 - lon0 + 1)
  if (tileCount > 80) return

  let i = 0
  for (let [x, y] of shuffle(range2D(lat0, lat1, lon0, lon1))) {
    x = `${x.toFixed(0).padStart(5, '0')}`
    y = `${y.toFixed(0).padStart(5, '0')}`
    if (!tileData[x + y]) {
      tileData[x + y] = fetch(`${ppTilePrefix}${x + y}.jsonl`)
        .then(r => r.text())
        .then(text => text.split('\n').slice(0, -1))
        .then(tile => tile.map(json => JSON.parse(json)))
        .catch(() => [])
        .then(tile => {
          const map = {}
          for (const v of tile) {
            const k =
              v['addr:housenumber'] +
              v['addr:housename'] +
              ':' +
              v['addr:street'] +
              ':' +
              v['addr:postcode']
            map[k] = v
          }
          return Object.values(map)
        })
    }
    tileData[x + y].then(async tile => {
      for (const v of tile) {
        if (drawn.has(v.id)) continue
        i++
        if (i % 100 === 0) {
          await new Promise(resolve => setTimeout(resolve, 16))
        }
        drawn.add(v.id)
        const nodes = []
        if (v.nodes.length > 1) {
          nodes.push(...v.nodes.map(p => [p.lat, p.lon]))
        } else {
          const {lat, lon} = v.nodes[0]
          nodes.push([lat - 0.00005, lon - 0.00008])
          nodes.push([lat + 0.00005, lon - 0.00008])
          nodes.push([lat + 0.00005, lon + 0.00008])
          nodes.push([lat - 0.00005, lon + 0.00008])
        }
        L.polygon(nodes, {
          stroke: false,
          fill: true,
          fillOpacity: 1,
          color: palette(v.price_adjusted_2019 / MAX)
        }).addTo(map)
      }
    })
  }
}

async function doMap(el) {
  const L = await import('leaflet')
  const map = L.map(el, {preferCanvas: true}).setView([51.451, -0.97], 15)

  const uri = 'https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png'
  L.tileLayer(uri, {opacity: 0.3}).addTo(map)

  return map
}

function PPMap() {
  const ref = useRef()
  useEffect(() => {
    const map = doMap(ref.current)
    const interval = setInterval(() => doPricePaid(map), 1000)
    return () => clearInterval(interval)
  }, [])

  return <div ref={ref} style={{flexGrow: 1, background: '#000'}}></div>
}

function Tick({p, v}) {
  return (
    <div
      style={{
        position: 'absolute',
        top: 0,
        left: (p * 100).toFixed(2) + '%',
        height: 40,
        width: 0,
        background: 'transparent',
        borderLeft: '2px solid #333',
        display: 'flex',
        justifyContent: 'center'
      }}
    >
      <div
        style={{
          position: 'absolute',
          flexShrink: 0,
          bottom: -20
        }}
      >
        ${v.toLocaleString()}
      </div>
    </div>
  )
}

function App() {
  const [width, setWidth] = useState()
  useEffect(() => {
    const onResize = () => setWidth(window.innerWidth)
    setWidth(window.innerWidth)
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])
  if (!width) return null
  const ticks = width > 1000 ? 11 : width > 500 ? 6 : 3
  return (
    <div style={{height: '100vh', display: 'flex', flexDirection: 'column'}}>
      <PPMap />
      <div style={{padding: '30px 50px', height: 60}}>
        <div style={{position: 'relative'}}>
          <div style={{background: gradient, height: 30}}></div>
          {Array(ticks)
            .fill()
            .map((_, i) => {
              const p = i / (ticks - 1)
              return <Tick p={p} v={MAX * p} />
            })}
        </div>
      </div>
    </div>
  )
}

export default App
