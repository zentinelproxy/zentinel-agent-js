// Smoke-test script: block /admin and everything under it, allow the rest
// and tag what is allowed.
function on_request_headers(request) {
    var uri = request.uri;
    if (uri === "/admin" || uri.startsWith("/admin/") || uri.startsWith("/admin?")) {
        return { decision: "block", status: 403, body: "blocked by js" };
    }
    return { decision: "allow", add_request_headers: { "X-Js-Smoke": "1" } };
}
