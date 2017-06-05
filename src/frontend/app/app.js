var game = new NGN.DATA.Model({
  autoid: true,
  fields: {
    state: {
      type: String
    },
    player: {
      type: String
    },
    watching: {
      type: Date
    },
    playing: {
      type: Date
    }
  },
  virtuals: {
    fullName: function () {
      return this.first + ' ' + this.last
    },
    age: function (e) {
      return moment().diff(moment(this.dob), 'years') // eslint-disable-line no-undef
    }
  }
});

var player = new NGN.DATA.Model({
  autoid: true,
  fields: {
    state: {
      type: String
    },
    name: {
      type: String
    }
  },
  virtuals: {
    fullName: function () {
      return this.first + ' ' + this.last
    },
    age: function (e) {
      return moment().diff(moment(this.dob), 'years') // eslint-disable-line no-undef
    }
  }
});