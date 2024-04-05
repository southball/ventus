import http from "k6/http";

export const options = {
  vus: 1,
  duration: "10m",
};

export default function () {
  http.post("http://host.docker.internal:3000", "Test");
}
