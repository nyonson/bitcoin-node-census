/**
 * Bitcoin Node Census Dashboard JavaScript
 * 
 * Loads census data from JSONL and populates existing DOM elements.
 * Uses shared Base16 Ashes theme from assets.yonson.dev.
 */

import * as echarts from 'https://cdn.jsdelivr.net/npm/echarts@6.0.0/dist/echarts.esm.js';
import { DEFAULT_OPTIONS, DEFAULT_LINE_STYLE } from 'https://assets.yonson.dev/v1/js/echarts-base16-ashes.js';

/**
 * Load and parse census data from JSONL file
 */
async function loadCensusData() {
    try {
        const response = await fetch('census.jsonl');
        if (!response.ok) {
            throw new Error('Could not load census.jsonl');
        }
        
        const text = await response.text();
        const lines = text.trim().split('\n').filter(line => line);
        const data = lines.map(line => JSON.parse(line));
        
        if (data.length === 0) {
            throw new Error('No data found in census.jsonl');
        }
        
        updateStats(data);
        initCharts(data);
    } catch (error) {
        document.getElementById('last-run').textContent = `Error: ${error.message}`;
    }
}

/**
 * Update stat values in existing DOM elements
 */
function updateStats(data) {
    // Sort by timestamp
    data.sort((a, b) => a.timestamp - b.timestamp);
    
    // Get latest data for stats
    const latest = data[data.length - 1];
    const stats = latest.stats;
    
    // Calculate percentages
    const filterPct = (stats.compact_filters / stats.total_nodes * 100).toFixed(1);
    const v2Pct = (stats.v2_transport / stats.total_nodes * 100).toFixed(1);
    const bothPct = (stats.v2_and_filters / stats.total_nodes * 100).toFixed(1);
    
    // Sort percentage stats by value (highest first)
    const percentageStats = [
        { value: filterPct, label: 'Compact Filters' },
        { value: v2Pct, label: 'V2 Transport' },
        { value: bothPct, label: 'Compact Filters & V2 Transport' }
    ].sort((a, b) => parseFloat(b.value) - parseFloat(a.value));
    
    // Format and update date
    const date = new Date(latest.timestamp * 1000);
    const formattedDate = date.toLocaleDateString('en-US', { 
        year: 'numeric', 
        month: 'long', 
        day: 'numeric'
    });
    
    // Update DOM elements
    document.getElementById('last-run').textContent = `Last run ${formattedDate}`;
    document.getElementById('total-nodes').textContent = stats.total_nodes.toLocaleString();
    document.getElementById('total-contacted').textContent = latest.total_contacted.toLocaleString();
    
    // Update percentage stats (sorted by highest value)
    document.getElementById('stat-1-value').textContent = `${percentageStats[0].value}%`;
    document.getElementById('stat-1-label').textContent = percentageStats[0].label;
    
    document.getElementById('stat-2-value').textContent = `${percentageStats[1].value}%`;
    document.getElementById('stat-2-label').textContent = percentageStats[1].label;
    
    document.getElementById('stat-3-value').textContent = `${percentageStats[2].value}%`;
    document.getElementById('stat-3-label').textContent = percentageStats[2].label;
}

/**
 * Initialize both charts and connect them for synchronization
 */
function initCharts(data) {
    const nodesChart = initNodesChart(data);
    const featuresChart = initFeaturesChart(data);
    
    // Connect charts for synchronized tooltips and brush selection
    echarts.connect([nodesChart, featuresChart]);
}

/**
 * Initialize nodes chart showing listening nodes over time
 */
function initNodesChart(data) {
    const chartDom = document.getElementById('nodes-chart');
    const chart = echarts.init(chartDom);
    
    // Prepare data for time series
    const nodeData = data.map(entry => [
        entry.timestamp * 1000, // Convert to milliseconds
        entry.stats.total_nodes
    ]);
    
    const option = {
        ...DEFAULT_OPTIONS,
        xAxis: {
            ...DEFAULT_OPTIONS.xAxis,
            type: 'time'
        },
        yAxis: {
            ...DEFAULT_OPTIONS.yAxis,
            type: 'value',
            name: 'Listening Nodes'
        },
        series: [{
            ...DEFAULT_LINE_STYLE,
            name: 'Listening Nodes',
            data: nodeData,
        }],
    };
    
    chart.setOption(option);
    
    return chart;
}

/**
 * Initialize features chart showing feature adoption percentages over time
 */
function initFeaturesChart(data) {
    const chartDom = document.getElementById('features-chart');
    const chart = echarts.init(chartDom);
    
    // Prepare data for each feature line
    const v2TransportData = data.map(entry => [
        entry.timestamp * 1000,
        (entry.stats.v2_transport / entry.stats.total_nodes * 100).toFixed(1)
    ]);
    
    const compactFiltersData = data.map(entry => [
        entry.timestamp * 1000,
        (entry.stats.compact_filters / entry.stats.total_nodes * 100).toFixed(1)
    ]);
    
    const bothFeaturesData = data.map(entry => [
        entry.timestamp * 1000,
        (entry.stats.v2_and_filters / entry.stats.total_nodes * 100).toFixed(1)
    ]);
    
    const option = {
        ...DEFAULT_OPTIONS,
        xAxis: {
            ...DEFAULT_OPTIONS.xAxis,
            type: 'time'
        },
        yAxis: {
            ...DEFAULT_OPTIONS.yAxis,
            type: 'value',
            name: 'Adoption %',
            axisLabel: {
                formatter: '{value}%'
            }
        },
        series: [
            {
                ...DEFAULT_LINE_STYLE,
                name: 'V2 Transport',
                data: v2TransportData,
            },
            {
                ...DEFAULT_LINE_STYLE,
                name: 'Compact Filters',
                data: compactFiltersData,
            },
            {
                ...DEFAULT_LINE_STYLE,
                name: 'Both Features',
                data: bothFeaturesData,
            }
        ],
    };
    
    chart.setOption(option);
    
    return chart;
}

// Initialize when page loads
document.addEventListener('DOMContentLoaded', loadCensusData);
