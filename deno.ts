import {Vm} from 'klaver:vm';


const vm = await Vm.open();

const ans = await vm.eval("42")
console.log(ans)
