/*
 * Copyright [2022] [Kevin Velasco]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

export function draw(nodes, links, selected, trans, canvas, search, width, height, highlightedNode, renderEdges) {
    const context = canvas.getContext("2d");
    context.save();
    context.clearRect(0, 0, width, height); // Clear the canvas.
    if (trans) {
        context.translate(trans.x, trans.y);
        context.scale(trans.k, trans.k);
    }
    if (renderEdges) {
        links.forEach(link => {

            context.beginPath();
            context.lineWidth = 0.25;
            context.strokeStyle = 'rgba(0,0,0,0.2)'
            if (link.async) {
                context.strokeStyle = "blue"
            }

            if (link.target.depth && link.source.depth) {
                context.lineWidth = 0.5 / link.target.depth;
                context.strokeStyle = `rgba(0, 0, 255, 1)`
            } else if (selected[link.source.id]) {
                context.lineWidth = 0.25;
                context.strokeStyle = "indigo"
            } else if (selected[link.target.id]) {
                context.lineWidth = 0.25;
                context.strokeStyle = "red"
            } else if (Object.keys(selected).length > 0) {
                context.strokeStyle = 'rgba(0,0,0,0)'
            }

            context.moveTo(link.source.x, link.source.y);
            context.lineTo(link.target.x, link.target.y);
            context.stroke();

        })
    }
    nodes.forEach(n => {
        context.beginPath();
        context.moveTo(n.x + 3, n.y);
        context.arc(n.x, n.y, 3, 0, 2 * Math.PI);

        context.fillStyle = n.color;

        if (selected[n.id] || highlightedNode === n.id || (search != null && search !== "" && n.label.includes(search))) {
            context.beginPath();
            context.moveTo(n.x + 3, n.y);
            context.lineWidth = 2;
            context.strokeStyle = 'rgba(0,0,0,1)'
            context.arc(n.x, n.y, 5, 0, 2 * Math.PI);
            context.stroke();
        } else if (Object.keys(selected).length > 0) {
            context.globalAlpha = 0.25;
        }


        context.fill();
        context.globalAlpha = 1
    });


    context.restore();

}
