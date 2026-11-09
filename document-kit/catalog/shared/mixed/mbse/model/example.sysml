package Example {
  part def Sensor;
  part def Controller;
  part def WeatherService;

  part sensorHub: Sensor;
  part controller: Controller;
  part weatherService: WeatherService;

  view def SystemContext;
  view def InteractionFlow;
}
