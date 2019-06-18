'use strict';

var rpc = {
    invoke : function(arg) { window.external.invoke(JSON.stringify(arg)); },
    init : function() { rpc.invoke({cmd : 'init'}); },
    log : function() {
        var s = '';
        for (var i = 0; i < arguments.length; i++) {
            if (i != 0) {
        s = s + ' ';
            }
            s = s + JSON.stringify(arguments[i]);
        }
        rpc.invoke({cmd : 'log', text : s});
    },
    search: function(query) { rpc.invoke({cmd: 'search', query: query}); },
    render: function(results) { 
        var resultsString = results
            .map(result => JSON.parse(result))
            .map(result => `${result.title[0]} found at ${result.location}\r\n`);
        document.getElementById('results').innerHTML = resultsString; 
    }
}

function search(ele) {
    if(event.key === 'Enter') {
        rpc.search(ele.value);        
    }
}

window.onload = function() { rpc.init(); };