
        var XX_SS = 3;
        try {
            var __ssq = new URLSearchParams(location.search).get("ss");
            if (__ssq) XX_SS = Math.max(1, Math.min(4, parseInt(__ssq, 10) || 1));
        } catch (e) {}
        (function () {
            try {
                var c = document.getElementById("glcanvas");
                var cap = function (v) { return Math.min(4096, Math.max(1, Math.round(v))); };
                // clientWidth/clientHeight 定义在 Element.prototype（部分内核在更
                // 深层）——沿原型链找原生 getter，找不到则维持 v1.9.9 尺寸不硬来
                var findGetter = function (name) {
                    var p = c;
                    while (p) {
                        var d = Object.getOwnPropertyDescriptor(p, name);
                        if (d && d.get) return d.get;
                        p = Object.getPrototypeOf(p);
                    }
                    return null;
                };
                var gw = findGetter("clientWidth");
                var gh = findGetter("clientHeight");
                if (!gw || !gh) return;
                Object.defineProperty(c, "clientWidth", {
                    get: function () { return cap(gw.call(c) * XX_SS); },
                    configurable: true
                });
                Object.defineProperty(c, "clientHeight", {
                    get: function () { return cap(gh.call(c) * XX_SS); },
                    configurable: true
                });
            } catch (e) { /* 覆盖失败则维持 v1.9.9 尺寸，功能不受影响 */ }
        })();
    